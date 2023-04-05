use async_trait::async_trait;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandOptionType,
        interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
    },
    prelude::Context,
    utils::Color,
};
use siege_api::{game_models::Side, models::GeneralStatistics};

use crate::{commands::utils::ExtractEnumOption, formatting::FormatEmbedded, SiegeApi};

use super::{
    get_user_from_command_or_default, lookup_siege_player, AddUserOptionToCommand, CommandError,
    CommandHandler,
};

#[derive(Debug, Clone, Copy, strum::EnumString, strum::Display, strum::EnumIter)]
enum Sorting {
    Kd,
    WinRate,
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
                    .name("side")
                    .description("Side to show operators for")
                    .kind(CommandOptionType::String)
                    .add_string_choice(Side::Attacker, Side::Attacker)
                    .add_string_choice(Side::Defender, Side::Defender)
                    .required(true)
            })
            .create_option(|option| {
                option
                    .name("sorting")
                    .description("Field to sort the statistics by. Defaults to KD")
                    .kind(CommandOptionType::String)
                    .required(false);

                use strum::IntoEnumIterator;
                Sorting::iter().for_each(|sorting| {
                    option.add_string_choice(sorting, sorting);
                });

                option
            })
            .add_user_option()
    }

    async fn run(
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), CommandError> {
        let side = command
            .extract_enum_option::<Side>("side")
            .expect("required argument");
        let sorting = command
            .extract_enum_option("sorting")
            .unwrap_or(Sorting::Kd);

        let user = get_user_from_command_or_default(command);
        tracing::info!(
            "Showing all operators for {user} on {side} side, sorting by {sorting}",
            user = user.name,
        );

        let player_id = lookup_siege_player(ctx, command, user).await?;

        let operator_response = {
            let data = ctx.data.read().await;
            let siege_client = data
                .get::<SiegeApi>()
                .expect("Siege client is always registered");
            siege_client.get_operators(player_id).await.unwrap()
        };

        let mut operators = operator_response
            .get_operators(side.into())
            .map(|os| {
                os.iter()
                    .filter_map(|op| match op {
                        GeneralStatistics::Operator(op) => Some(op),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap();

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
        };

        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|msg| {
                        msg.embed(|embed| {
                            embed
                                .thumbnail(user.avatar_url().unwrap_or_default())
                                .title(format!("{} operator statistics for {}", side, user.name))
                                .color(Color::TEAL)
                                .format(&operators)
                        })
                    })
            })
            .await
            .map_err(CommandError::SerenityError)?;

        Ok(())
    }
}
