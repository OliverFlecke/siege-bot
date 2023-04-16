use async_trait::async_trait;
use serenity::{
    builder::{CreateApplicationCommand, CreateEmbed},
    model::prelude::{
        command::CommandOptionType, interaction::application_command::CommandDataOptionValue,
    },
    utils::Color,
};
use siege_api::{game_models::Side, models::OperatorStatistics};

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

                use strum::IntoEnumIterator;
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
            siege_client.get_operators(player_id).await.unwrap()
        };

        let mut operators = operator_response
            .get_operators(side.into())
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
                    .title(format!("{} operator statistics for {}", side, user.name))
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
    use serde_json::Value;

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
        assert_eq!(opt.get("name").unwrap(), "user");
        assert_eq!(*opt.get("required").unwrap(), Value::Bool(false));
        assert_eq!(opt.get("type").unwrap().as_u64().unwrap(), 6); // Corresponds to `CommandOptionType::Integer`
        assert!(!opt.get("description").unwrap().as_str().unwrap().is_empty());
    }
}
