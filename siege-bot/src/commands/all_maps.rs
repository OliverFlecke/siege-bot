use async_trait::async_trait;
use serenity::{
    builder::{CreateApplicationCommand, CreateEmbed},
    model::prelude::{
        command::CommandOptionType, interaction::application_command::CommandDataOptionValue,
    },
    utils::Color,
};
use siege_api::models::{MapStatistics, SideOrAll};
use strum::IntoEnumIterator;

use crate::{formatting::FormatEmbedded, SiegeApi};

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

static SIDE: &str = "side";
static SORTING: &str = "sorting";
static MINIMUM_ROUNDS: &str = "minimum_rounds";

pub struct AllMapsCommand;

#[async_trait]
impl CommandHandler for AllMapsCommand {
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name("all_maps")
            .description("List statistics for all maps for a given side or overall")
            .create_option(|option| {
                option
                    .name(SIDE)
                    .description("Side to show maps for")
                    .kind(CommandOptionType::String)
                    .required(true);

                SideOrAll::iter().for_each(|side| {
                    option.add_string_choice(side, side);
                });

                option
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
            .add_user_option()
    }

    async fn run<Ctx, Cmd>(ctx: &Ctx, command: &Cmd) -> CmdResult
    where
        Ctx: DiscordContext + Send + Sync,
        Cmd: DiscordAppCmd + 'static + Send + Sync,
    {
        let side = command
            .extract_enum_option::<SideOrAll>(SIDE)
            .expect("required argument");
        let sorting = command.extract_enum_option(SORTING).unwrap_or(Sorting::Kd);
        let minimum_rounds = command
            .get_option(MINIMUM_ROUNDS)
            .and_then(|x| match x {
                CommandDataOptionValue::Integer(value) => Some(value),
                _ => None,
            })
            .unwrap_or(0);

        let user = command.get_user_from_command_or_default();
        tracing::info!(
            "Showing all operators for {user} on {side} side, sorting by {sorting}",
            user = user.name,
        );

        let player_id = ctx.lookup_siege_player(command, &user).await?;

        let response = {
            let data = ctx.data().read().await;
            let siege_client = data
                .get::<SiegeApi>()
                .expect("Siege client is always registered");
            match siege_client.get_maps(player_id).await {
                Ok(data) => data,
                Err(err) => {
                    tracing::error!("Failed to fetch data: {err:?}");
                    return command.send_text(ctx.http(), "Failed to fetch data").await;
                }
            }
        };

        let mut maps = response
            .get_maps(side)
            .iter()
            .filter(|x| *x.statistics().rounds_played() as i64 >= minimum_rounds)
            .copied()
            .collect::<Vec<_>>();

        sort(&mut maps, sorting);

        command
            .send_embedded(
                ctx.http(),
                CreateEmbed::default()
                    .thumbnail(user.avatar_url().unwrap_or_default())
                    .title(format!("{} map statistics for {}", side, user.name))
                    .color(Color::TEAL)
                    .format(&maps)
                    .to_owned(),
            )
            .await
    }
}

fn sort(maps: &mut [&MapStatistics], sorting: Sorting) {
    match sorting {
        Sorting::Kd => {
            maps.sort_by(|a, b| {
                b.statistics()
                    .kill_death_ratio()
                    .partial_cmp(a.statistics().kill_death_ratio())
                    .expect("should always be valid")
            });
        }
        Sorting::WinRate => {
            maps.sort_by(|a, b| {
                b.statistics()
                    .rounds_win_rate()
                    .partial_cmp(&a.statistics().rounds_win_rate())
                    .expect("should always be valid")
            });
        }
        Sorting::RoundsPlayed => maps.sort_by(|a, b| {
            b.statistics()
                .rounds_played()
                .cmp(a.statistics().rounds_played())
        }),
    };
}

#[cfg(test)]
mod test {

    use mockall::predicate::*;
    use serde_json::Value;
    use serenity::model::user::User;
    use siege_api::models::StatisticResponse;
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
        let command = AllMapsCommand::register(&mut command);

        // Assert
        assert_eq!(command.0.get("name").unwrap(), "all_maps");
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
        assert_eq!(opt.get("choices").unwrap().as_array().unwrap().len(), 3);

        let opt = options.get(1).unwrap();
        assert_eq!(opt.get("name").unwrap(), SORTING);
        assert_eq!(*opt.get("required").unwrap(), Value::Bool(false));
        assert_eq!(opt.get("choices").unwrap().as_array().unwrap().len(), 3);

        let opt = options.get(2).unwrap();
        assert_eq!(opt.get("name").unwrap(), MINIMUM_ROUNDS);
        assert_eq!(*opt.get("required").unwrap(), Value::Bool(false));
        assert_eq!(opt.get("type").unwrap().as_u64().unwrap(), 4); // Corresponds to `CommandOptionType::Integer`

        let opt = options.get(3).unwrap();
        assert_eq!(opt.get("name").unwrap(), "user");
        assert_eq!(*opt.get("required").unwrap(), Value::Bool(false));
        assert_eq!(opt.get("type").unwrap().as_u64().unwrap(), 6); // Corresponds to `CommandOptionType::Integer`
        assert!(!opt.get("description").unwrap().as_str().unwrap().is_empty());
    }

    #[tokio::test]
    async fn validate_run() {
        let user = User::default();
        let siege_id = Uuid::new_v4();

        // Testing different combinations of arguments.
        for (side, sorting, rounds) in vec![
            (Some(SideOrAll::All), None, Some(10 as i64)),
            (Some(SideOrAll::Attacker), Some(Sorting::Kd), None),
            (Some(SideOrAll::Defender), Some(Sorting::RoundsPlayed), None),
            (Some(SideOrAll::Defender), Some(Sorting::WinRate), None),
        ] {
            let mut ctx = MockDiscordContext::new();
            ctx.expect_http().return_const(None);
            ctx.expect_lookup_siege_player::<MockDiscordAppCmd>()
                .with(always(), eq(user.clone()))
                .once()
                .returning(move |_, _| Ok(siege_id));

            let mut mock_client = MockSiegeClient::default();
            mock_client.expect_get_maps().once().returning(|_| {
                let content = std::fs::read_to_string("../samples/maps.json").unwrap();
                let stats: StatisticResponse = serde_json::from_str(content.as_str()).unwrap();
                Ok(stats)
            });
            register_client_in_type_map(&mut ctx, mock_client).await;

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
                .expect_get_user_from_command_or_default()
                .return_const(user.clone());

            // Assert the right message is sent back
            command
                .expect_send_embedded()
                .once()
                .with(always(), always())
                .returning(|_, _| Ok(()));

            // Act
            assert!(AllMapsCommand::run(&ctx, &command).await.is_ok());
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
            .expect_get_maps()
            .once()
            .returning(|_| Err(siege_api::auth::ConnectError::InvalidPassword));
        register_client_in_type_map(&mut ctx, mock_client).await;

        // Setup command
        let mut command = MockDiscordAppCmd::new();
        command
            .expect_extract_enum_option()
            .with(eq(SIDE))
            .return_const(SideOrAll::All);
        command
            .expect_extract_enum_option::<Sorting>()
            .with(eq(SORTING))
            .return_const(None);
        command
            .expect_get_option()
            .with(eq(MINIMUM_ROUNDS))
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
        assert!(AllMapsCommand::run(&ctx, &command).await.is_ok());
    }
}
