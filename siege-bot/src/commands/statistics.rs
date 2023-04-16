use async_trait::async_trait;
use serenity::{
    builder::{CreateApplicationCommand, CreateEmbed},
    utils::Color,
};

use crate::SiegeApi;

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
            .add_user_option()
    }

    async fn run<Ctx, Cmd>(ctx: &Ctx, command: &Cmd) -> CmdResult
    where
        Ctx: DiscordContext + Send + Sync,
        Cmd: DiscordAppCmd + 'static + Send + Sync,
    {
        let user = command.get_user_from_command_or_default();
        let player_id = ctx.lookup_siege_player(command, &user).await?;

        tracing::info!("Getting statistics for {}", user.name);

        let profiles = {
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

        let profile: siege_api::models::FullProfile = profiles[0];

        let season = profile.season_statistics().clone();
        let matches = season.match_outcomes().clone();

        command
            .send_embedded(
                ctx.http().clone(),
                CreateEmbed::default()
                    .title(format!("Statistics for {}", user.name))
                    .thumbnail(user.avatar_url().unwrap())
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
                    )
                    .color(Color::DARK_RED)
                    .clone(),
            )
            .await
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
        assert_eq!(opt.get("name").unwrap(), "user");
        assert_eq!(*opt.get("required").unwrap(), Value::Bool(false));
        assert_eq!(opt.get("type").unwrap().as_u64().unwrap(), 6); // Corresponds to `CommandOptionType::User`
        assert!(!opt.get("description").unwrap().as_str().unwrap().is_empty());
    }

    #[tokio::test]
    async fn validate_run() {
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
                Ok(stats.platform_families_full_profiles()[0]
                    .board_ids_full_profiles()
                    .iter()
                    .map(|x| x.full_profiles()[0])
                    .collect::<Vec<_>>())
            });
        register_client_in_type_map(&mut ctx, mock_client).await;

        let mut command = MockDiscordAppCmd::new();
        command
            .expect_get_user_from_command_or_default()
            .return_const(user.clone());
        command
            .expect_send_embedded()
            .once()
            .with(always(), always())
            .returning(|_, _| Ok(()));

        // Act
        assert!(StatisticsCommand::run(&ctx, &command).await.is_ok());
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
            .expect_send_text()
            .once()
            .with(always(), eq("Failed to fetch data"))
            .returning(|_, _| Ok(()));

        // Act
        assert!(StatisticsCommand::run(&ctx, &command).await.is_ok());
    }
}
