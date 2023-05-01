use async_trait::async_trait;
use serenity::{
    builder::{CreateApplicationCommand, CreateEmbed},
    model::prelude::command::CommandOptionType,
    utils::Color,
};
use siege_api::{
    data::rank::Rank,
    models::{GameMode, PlatformFamily},
};
use strum::IntoEnumIterator;

use crate::{
    constants::{GAME_MODE, PLATFORM},
    SiegeApi,
};

use super::{
    context::DiscordContext, discord_app_command::DiscordAppCmd, AddUserOptionToCommand, CmdResult,
    CommandHandler,
};

pub struct StatisticsCommand;

#[async_trait]
impl CommandHandler for StatisticsCommand {
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name("statistics")
            .description("Get the statistics for a Siege player")
            .create_option(|option| {
                option
                    .name(GAME_MODE)
                    .description("Game mode to get statistics for")
                    .kind(CommandOptionType::String)
                    .required(false);

                GameMode::iter().for_each(|x| {
                    option.add_string_choice(x.to_string(), x.to_string());
                });

                option
            })
            .create_option(|option| {
                option
                    .name(PLATFORM)
                    .description("Platform to retrieve player's data from")
                    .kind(CommandOptionType::String)
                    .required(false);

                PlatformFamily::iter().for_each(|x| {
                    option.add_string_choice(x.to_string(), x.to_string());
                });

                option
            })
            .add_user_option()
    }

    async fn run<Ctx, Cmd>(ctx: &Ctx, command: &Cmd) -> CmdResult
    where
        Ctx: DiscordContext + Send + Sync,
        Cmd: DiscordAppCmd + 'static + Send + Sync,
    {
        let user = command.get_user_from_command_or_default();
        let player_id = ctx.lookup_siege_player(command, &user).await?;
        let platform = command
            .extract_enum_option(PLATFORM)
            .unwrap_or(PlatformFamily::Pc);
        let game_mode = command
            .extract_enum_option(GAME_MODE)
            .unwrap_or(GameMode::Casual);

        tracing::info!("Getting statistics for {}", user.name);

        let statistics = {
            let data = ctx.data().read().await;
            let siege_client = data
                .get::<SiegeApi>()
                .expect("Siege client is always registered");
            match siege_client.get_full_profiles(player_id).await {
                Ok(data) => data,
                Err(err) => {
                    tracing::error!("Failed to fetch data: {err:?}");
                    return command.send_text(ctx.http(), "Failed to fetch data").await;
                }
            }
        };

        match statistics.get_board(platform, game_mode) {
            Some(data) => {
                let season = *data.season_statistics();
                let matches = *season.match_outcomes();

                let mut embedded = CreateEmbed::default();
                embedded.title(format!("{game_mode} statistics for {} | {}", user.name, data.profile().season()))
                        .thumbnail(user.avatar_url().unwrap())
                        .color(Color::DARK_RED)
                        .field(
                            "Kill/death",
                            format!(
                                "K/D: **{kd:.2}** - Kills {kills} / Deaths {deaths}",
                                kd = season.kd(),
                                kills = season.kills(),
                                deaths = season.deaths(),
                            ),
                            false,
                        )
                        .field(
                            "Match",
                            format!(
                                "Matches {total} - **{win_rate:.2} %** - Wins **{wins}** / Losses **{losses}**",
                                total = matches.total_matches(),
                                wins = matches.wins(),
                                losses = matches.losses(),
                                win_rate = matches.win_rate() * 100.0,
                            ),
                            false,
                        );

                if game_mode == GameMode::Ranked {
                    embedded.field(
                        "Rank",
                        format!(
                            "Max rank: **{}**\nMax rank points: **{}**",
                            Rank::from_repr(*data.profile().max_rank()).unwrap_or(Rank::Unranked),
                            data.profile().max_rank_points(),
                        ),
                        false,
                    );
                }

                command.send_embedded(ctx.http().clone(), embedded).await
            }
            None => {
                command
                    .send_text(
                        ctx.http().clone(),
                        format!(
                            "No data found for {game_mode}/{platform} for player {}",
                            user.tag()
                        )
                        .as_str(),
                    )
                    .await
            }
        }
    }
}

#[cfg(test)]
mod test {
    use mockall::predicate::*;
    use serde_json::Value;
    use serenity::model::user::User;
    use siege_api::models::RankedV2Response;
    use uuid::Uuid;

    use crate::commands::{
        context::MockDiscordContext,
        discord_app_command::MockDiscordAppCmd,
        test::{register_client_in_type_map, MockSiegeClient},
    };

    use super::*;

    #[tokio::test]
    async fn validate_register() {
        let mut command = CreateApplicationCommand::default();
        let command = StatisticsCommand::register(&mut command);

        // Assert
        assert_eq!(command.0.get("name").unwrap(), "statistics");
        assert!(!command
            .0
            .get("description")
            .and_then(|x| x.as_str())
            .unwrap()
            .is_empty());

        let options = command.0.get("options").unwrap().as_array().unwrap();

        let opt = options.get(0).unwrap();
        assert_eq!(opt.get("name").unwrap(), GAME_MODE);
        assert_eq!(*opt.get("required").unwrap(), Value::Bool(false));
        assert_eq!(opt.get("type").unwrap().as_u64().unwrap(), 3);
        assert!(!opt.get("description").unwrap().as_str().unwrap().is_empty());
        assert_eq!(opt.get("choices").unwrap().as_array().unwrap().len(), 4);

        let opt = options.get(1).unwrap();
        assert_eq!(opt.get("name").unwrap(), PLATFORM);
        assert_eq!(*opt.get("required").unwrap(), Value::Bool(false));
        assert_eq!(opt.get("type").unwrap().as_u64().unwrap(), 3);
        assert!(!opt.get("description").unwrap().as_str().unwrap().is_empty());
        assert_eq!(opt.get("choices").unwrap().as_array().unwrap().len(), 2);

        let opt = options.get(2).unwrap();
        assert_eq!(opt.get("name").unwrap(), "user");
        assert_eq!(*opt.get("required").unwrap(), Value::Bool(false));
        assert_eq!(opt.get("type").unwrap().as_u64().unwrap(), 6); // Corresponds to `CommandOptionType::User`
        assert!(!opt.get("description").unwrap().as_str().unwrap().is_empty());
    }

    #[tokio::test]
    async fn validate_run() {
        let user = User::default();
        let siege_id = Uuid::new_v4();

        for game_mode in vec![
            GameMode::Casual,
            GameMode::Event,
            GameMode::Ranked,
            GameMode::Warmup,
        ] {
            let mut ctx = MockDiscordContext::new();
            ctx.expect_http().return_const(None);
            ctx.expect_lookup_siege_player::<MockDiscordAppCmd>()
                .with(always(), eq(user.clone()))
                .once()
                .returning(move |_, _| Ok(siege_id));

            let mut mock_client = MockSiegeClient::default();
            mock_client
                .expect_get_full_profiles()
                .once()
                .returning(|_| {
                    let content = std::fs::read_to_string("../samples/full_profile.json").unwrap();
                    let stats: RankedV2Response = serde_json::from_str(content.as_str()).unwrap();
                    Ok(stats)
                });
            register_client_in_type_map(&mut ctx, mock_client).await;

            let mut command = MockDiscordAppCmd::new();
            command
                .expect_get_user_from_command_or_default()
                .return_const(user.clone());
            command
                .expect_extract_enum_option()
                .with(eq(GAME_MODE))
                .return_once(move |_| Some(game_mode));
            command
                .expect_extract_enum_option()
                .with(eq(PLATFORM))
                .return_once(|_| Some(PlatformFamily::Pc));

            command
                .expect_send_embedded()
                .once()
                .with(always(), always())
                .returning(|_, _| Ok(()));

            // Act
            assert!(StatisticsCommand::run(&ctx, &command).await.is_ok());
        }
    }

    #[tokio::test]
    async fn validate_run_api_failed() {
        let user = User::default();
        let siege_id = Uuid::new_v4();

        let mut ctx = MockDiscordContext::new();
        ctx.expect_http().return_const(None);
        ctx.expect_lookup_siege_player::<MockDiscordAppCmd>()
            .with(always(), eq(user.clone()))
            .once()
            .returning(move |_, _| Ok(siege_id));

        let mut mock_client = MockSiegeClient::default();
        mock_client
            .expect_get_full_profiles()
            .once()
            .returning(|_| Err(siege_api::auth::ConnectError::InvalidPassword));
        register_client_in_type_map(&mut ctx, mock_client).await;

        let mut command = MockDiscordAppCmd::new();
        command
            .expect_get_user_from_command_or_default()
            .return_const(user.clone());
        command
            .expect_extract_enum_option()
            .with(eq(GAME_MODE))
            .return_once(move |_| Some(GameMode::Casual));
        command
            .expect_extract_enum_option()
            .with(eq(PLATFORM))
            .return_once(|_| Some(PlatformFamily::Pc));

        command
            .expect_send_text()
            .once()
            .with(always(), eq("Failed to fetch data"))
            .returning(|_, _| Ok(()));

        // Act
        assert!(StatisticsCommand::run(&ctx, &command).await.is_ok());
    }

    #[tokio::test]
    async fn validate_run_no_statistics_for_choices() {
        let user = User::default();
        let siege_id = Uuid::new_v4();

        let mut ctx = MockDiscordContext::new();
        ctx.expect_http().return_const(None);
        ctx.expect_lookup_siege_player::<MockDiscordAppCmd>()
            .with(always(), eq(user.clone()))
            .once()
            .returning(move |_, _| Ok(siege_id));

        let mut mock_client = MockSiegeClient::default();
        mock_client
            .expect_get_full_profiles()
            .once()
            .returning(|_| {
                let content = std::fs::read_to_string("../samples/full_profile.json").unwrap();
                let stats: RankedV2Response = serde_json::from_str(content.as_str()).unwrap();
                Ok(stats)
            });
        register_client_in_type_map(&mut ctx, mock_client).await;

        let mut command = MockDiscordAppCmd::new();
        command
            .expect_get_user_from_command_or_default()
            .return_const(user.clone());
        command
            .expect_extract_enum_option()
            .with(eq(GAME_MODE))
            .return_once(move |_| Some(GameMode::Casual));
        command
            .expect_extract_enum_option()
            .with(eq(PLATFORM))
            .return_once(|_| Some(PlatformFamily::Console));

        command
            .expect_send_text()
            .once()
            .with(
                always(),
                eq(format!(
                    "No data found for {}/{} for player {}",
                    GameMode::Casual,
                    PlatformFamily::Console,
                    user.tag()
                )),
            )
            .returning(|_, _| Ok(()));

        // Act
        assert!(StatisticsCommand::run(&ctx, &command).await.is_ok());
    }
}
