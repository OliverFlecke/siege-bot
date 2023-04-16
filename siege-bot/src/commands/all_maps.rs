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
            siege_client.get_maps(player_id).await.unwrap()
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
    use std::sync::Arc;

    use mockall::predicate::*;
    use serde_json::Value;
    use serenity::{
        model::user::User,
        prelude::{RwLock, TypeMap},
    };
    use uuid::Uuid;

    use crate::{
        commands::{
            context::MockDiscordContext, discord_app_command::MockDiscordAppCmd,
            test::MockSiegeClient,
        },
        siege_player_lookup::{MockPlayerLookup, SiegePlayerLookup},
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
    #[ignore]
    async fn validate_run() {
        let user = User::default();
        let siege_id = Uuid::new_v4();
        let ubisoft_name = "some_name".to_string();

        // Ensure the expected message is sent back through the command
        let mut ctx = MockDiscordContext::new();
        let mut mock_client = MockSiegeClient::default();
        mock_client
            .expect_search_for_player()
            .with(eq(ubisoft_name.clone()))
            .once()
            .return_once(move |_| Ok(siege_id));

        // Ensure the right user/id pair is inserted into the lookup.
        let mock_lookup = MockPlayerLookup::default();
        // mock_lookup
        //     .expect_get()
        //     .with(always(), eq(siege_id))
        //     .once()
        //     .return_once(|_, _| Ok(()));

        // Setup lookup
        let data = Arc::new(RwLock::new(TypeMap::default()));
        {
            let mut data = data.write().await;
            data.insert::<SiegeApi>(Arc::new(mock_client));
            data.insert::<SiegePlayerLookup>(Arc::new(RwLock::new(mock_lookup)));
        }
        ctx.expect_data().return_const(data);

        // Arrange command
        let mut command = MockDiscordAppCmd::new();
        command
            .expect_get_option()
            .with(eq(SIDE))
            .return_once(move |_| Some(CommandDataOptionValue::String(SideOrAll::All.to_string())));
        command
            .expect_get_user_from_command_or_default()
            .return_once(|| user);

        // Assert the right message is sent back
        command
            .expect_send_text()
            .once()
            .with(always(), eq("Accounts linked!"))
            .returning(|_, _| Ok(()));

        // Act
        assert!(AllMapsCommand::run(&ctx, &command).await.is_ok());
    }
}
