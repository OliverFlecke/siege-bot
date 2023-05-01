use async_trait::async_trait;
use serenity::{
    builder::{CreateApplicationCommand, CreateEmbed},
    model::prelude::{
        command::CommandOptionType, interaction::application_command::CommandDataOptionValue,
    },
    utils::Color,
};
use siege_api::{
    game_models::Side,
    models::{AllOrRanked, OperatorStatistics},
};
use strum::IntoEnumIterator;

use crate::{
    constants::{GAME_MODE, MINIMUM_ROUNDS, SIDE, SORTING},
    formatting::FormatEmbedded,
    SiegeApi,
};

use super::{
    context::DiscordContext, discord_app_command::DiscordAppCmd, AddUserOptionToCommand, CmdResult,
    CommandHandler,
};

#[derive(Debug, Clone, Copy, strum::EnumString, strum::Display, strum::EnumIter)]
enum Sorting {
    Kd,
    WinRate,
    RoundsPlayed,
}

pub struct AllOperatorCommand;

#[async_trait]
impl CommandHandler for AllOperatorCommand {
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name("all_operators")
            .description("List statistics for all operators for a given side")
            .create_option(|option| {
                option
                    .name(SIDE)
                    .description("Side to show operators for")
                    .kind(CommandOptionType::String)
                    .add_string_choice(Side::Attacker, Side::Attacker)
                    .add_string_choice(Side::Defender, Side::Defender)
                    .required(true)
            })
            .create_option(|option| {
                option
                    .name(SORTING)
                    .description("Field to sort the statistics by. Defaults to KD")
                    .kind(CommandOptionType::String)
                    .required(false);

                Sorting::iter().for_each(|sorting| {
                    option.add_string_choice(sorting, sorting);
                });

                option
            })
            .create_option(|option| {
                option
                    .name(MINIMUM_ROUNDS)
                    .description("Ignore operators you have played for less than this limit")
                    .kind(CommandOptionType::Integer)
                    .required(false)
            })
            .create_option(|option| {
                option
                    .name(GAME_MODE)
                    .description("Game mode to retreive statistics for")
                    .kind(CommandOptionType::String)
                    .required(false);

                AllOrRanked::iter().for_each(|mode| {
                    option.add_string_choice(mode, mode);
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
        let side = command
            .extract_enum_option::<Side>(SIDE)
            .expect("required argument");
        let sorting = command.extract_enum_option(SORTING).unwrap_or(Sorting::Kd);
        let minimum_rounds = command
            .get_option(MINIMUM_ROUNDS)
            .and_then(|x| match x {
                CommandDataOptionValue::Integer(value) => Some(value),
                _ => None,
            })
            .unwrap_or(0);
        let game_mode = command
            .extract_enum_option(GAME_MODE)
            .unwrap_or(AllOrRanked::All);

        let user = command.get_user_from_command_or_default();
        tracing::info!(
            "Showing all operators for {user} on {side} side, sorting by {sorting}",
            user = user.name,
        );

        let player_id = ctx.lookup_siege_player(command, &user).await?;

        let operator_response = {
            let data = ctx.data().read().await;
            let siege_client = data
                .get::<SiegeApi>()
                .expect("Siege client is always registered");
            match siege_client.get_operators(player_id).await {
                Ok(data) => data,
                Err(err) => {
                    tracing::error!("Failed to fetch data: {err:?}");
                    return command.send_text(ctx.http(), "Failed to fetch data").await;
                }
            }
        };

        let mut operators = operator_response
            .get_operators(game_mode, side.into())
            .iter()
            .filter(|op| *op.statistics().rounds_played() as i64 >= minimum_rounds)
            .copied()
            .collect::<Vec<_>>();
        sort(&mut operators, sorting);

        command
            .send_embedded(
                ctx.http().clone(),
                CreateEmbed::default()
                    .thumbnail(user.avatar_url().unwrap_or_default())
                    .title(format!(
                        "{}/{} operator statistics for {}",
                        game_mode, side, user.name
                    ))
                    .color(Color::TEAL)
                    .format(&operators)
                    .to_owned(),
            )
            .await
    }
}

fn sort(operators: &mut [&OperatorStatistics], sorting: Sorting) {
    match sorting {
        Sorting::Kd => {
            operators.sort_by(|a, b| {
                b.statistics()
                    .kill_death_ratio()
                    .partial_cmp(a.statistics().kill_death_ratio())
                    .expect("should always be valid")
            });
        }
        Sorting::WinRate => {
            operators.sort_by(|a, b| {
                b.statistics()
                    .rounds_win_rate()
                    .partial_cmp(&a.statistics().rounds_win_rate())
                    .expect("should always be valid")
            });
        }
        Sorting::RoundsPlayed => operators.sort_by(|a, b| {
            b.statistics()
                .rounds_played()
                .cmp(a.statistics().rounds_played())
        }),
    };
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use mockall::predicate::*;
    use serde_json::Value;
    use serenity::{
        model::user::User,
        prelude::{RwLock, TypeMap},
    };
    use siege_api::models::StatisticResponse;
    use uuid::Uuid;

    use crate::{
        commands::{
            context::MockDiscordContext,
            discord_app_command::MockDiscordAppCmd,
            test::{register_client_in_type_map, MockSiegeClient},
        },
        constants::USER,
        siege_player_lookup::{MockPlayerLookup, SiegePlayerLookup},
    };

    use super::*;

    #[tokio::test]
    async fn validate_register() {
        let mut command = CreateApplicationCommand::default();
        let command = AllOperatorCommand::register(&mut command);

        // Assert
        assert_eq!(command.0.get("name").unwrap(), "all_operators");
        assert!(!command
            .0
            .get("description")
            .and_then(|x| x.as_str())
            .unwrap()
            .is_empty());

        let options = command.0.get("options").unwrap().as_array().unwrap();
        // Assert first options
        let opt = options.get(0).unwrap();
        assert_eq!(opt.get("name").unwrap(), SIDE);
        assert_eq!(*opt.get("required").unwrap(), Value::Bool(true));
        assert_eq!(opt.get("choices").unwrap().as_array().unwrap().len(), 2);

        let opt = options.get(1).unwrap();
        assert_eq!(opt.get("name").unwrap(), SORTING);
        assert_eq!(*opt.get("required").unwrap(), Value::Bool(false));
        assert_eq!(opt.get("choices").unwrap().as_array().unwrap().len(), 3);

        let opt = options.get(2).unwrap();
        assert_eq!(opt.get("name").unwrap(), MINIMUM_ROUNDS);
        assert_eq!(*opt.get("required").unwrap(), Value::Bool(false));
        assert_eq!(opt.get("type").unwrap().as_u64().unwrap(), 4); // Corresponds to `CommandOptionType::Integer`

        let opt = options.get(3).unwrap();
        assert_eq!(opt.get("name").unwrap(), GAME_MODE);
        assert_eq!(opt.get("type").unwrap().as_u64().unwrap(), 3); // Corresponds to `CommandOptionType::String`
        assert_eq!(*opt.get("required").unwrap(), Value::Bool(false));
        assert_eq!(opt.get("choices").unwrap().as_array().unwrap().len(), 2);

        let opt = options.get(4).unwrap();
        assert_eq!(opt.get("name").unwrap(), USER);
        assert_eq!(*opt.get("required").unwrap(), Value::Bool(false));
        assert_eq!(opt.get("type").unwrap().as_u64().unwrap(), 6); // Corresponds to `CommandOptionType::Integer`
        assert!(!opt.get("description").unwrap().as_str().unwrap().is_empty());
    }

    #[tokio::test]
    async fn validate_run() {
        let user = User::default();
        let siege_id = Uuid::new_v4();

        // Testing different combinations of arguments.
        for (side, sorting, rounds, game_mode) in vec![
            (
                Some(Side::Attacker),
                Some(Sorting::Kd),
                Some(10 as i64),
                Some(AllOrRanked::All),
            ),
            (
                Some(Side::Defender),
                Some(Sorting::RoundsPlayed),
                None,
                Some(AllOrRanked::Ranked),
            ),
            (Some(Side::Defender), Some(Sorting::WinRate), None, None),
        ] {
            // Ensure the expected message is sent back through the command
            let mut ctx = MockDiscordContext::new();
            ctx.expect_http().return_const(None);
            ctx.expect_lookup_siege_player::<MockDiscordAppCmd>()
                .with(always(), eq(user.clone()))
                .once()
                .returning(move |_, _| Ok(siege_id));

            let mut mock_client = MockSiegeClient::default();
            mock_client.expect_get_operators().once().returning(|_| {
                let content = std::fs::read_to_string("../samples/operators.json").unwrap();
                let stats: StatisticResponse = serde_json::from_str(content.as_str()).unwrap();
                Ok(stats)
            });

            // Ensure the right user/id pair is inserted into the lookup.
            let mock_lookup = MockPlayerLookup::default();

            // Setup lookup
            let data = Arc::new(RwLock::new(TypeMap::default()));
            {
                let mut data = data.write().await;
                data.insert::<SiegeApi>(Arc::new(mock_client));
                data.insert::<SiegePlayerLookup>(Arc::new(RwLock::new(mock_lookup)));
            }
            ctx.expect_data().return_const(data);

            let mut command = MockDiscordAppCmd::new();
            command
                .expect_extract_enum_option()
                .with(eq(SIDE))
                .return_once(move |_| side);
            command
                .expect_extract_enum_option::<Sorting>()
                .with(eq(SORTING))
                .return_const(sorting);
            command
                .expect_get_option()
                .with(eq(MINIMUM_ROUNDS))
                .return_const(rounds.map(|x| CommandDataOptionValue::Integer(x)));
            command
                .expect_extract_enum_option::<AllOrRanked>()
                .with(eq(GAME_MODE))
                .return_const(game_mode);
            command
                .expect_get_user_from_command_or_default()
                .return_const(user.clone());

            // Assert the right message is sent back
            command
                .expect_send_embedded()
                .once()
                .with(always(), always())
                .returning(|_, _| Ok(()));

            // Act
            assert!(AllOperatorCommand::run(&ctx, &command).await.is_ok());
        }
    }

    #[tokio::test]
    async fn validate_run_api_failed() {
        let user = User::default();
        let siege_id = Uuid::new_v4();

        // Ensure the expected message is sent back through the command
        let mut ctx = MockDiscordContext::new();
        ctx.expect_http().return_const(None);
        ctx.expect_lookup_siege_player::<MockDiscordAppCmd>()
            .with(always(), eq(user.clone()))
            .once()
            .returning(move |_, _| Ok(siege_id));

        let mut mock_client = MockSiegeClient::default();
        mock_client
            .expect_get_operators()
            .once()
            .returning(|_| Err(siege_api::auth::ConnectError::InvalidPassword));
        register_client_in_type_map(&mut ctx, mock_client).await;

        let mut command = MockDiscordAppCmd::new();
        command
            .expect_extract_enum_option()
            .with(eq(SIDE))
            .return_const(Side::Attacker);
        command
            .expect_extract_enum_option::<Sorting>()
            .with(eq(SORTING))
            .return_const(None);
        command
            .expect_get_option()
            .with(eq(MINIMUM_ROUNDS))
            .return_const(None);
        command
            .expect_extract_enum_option::<AllOrRanked>()
            .with(eq(GAME_MODE))
            .return_const(None);
        command
            .expect_get_user_from_command_or_default()
            .return_const(user.clone());

        // Assert the right message is sent back
        command
            .expect_send_text()
            .once()
            .with(always(), eq("Failed to fetch data"))
            .returning(|_, _| Ok(()));

        // Act
        assert!(AllOperatorCommand::run(&ctx, &command).await.is_ok());
    }
}
