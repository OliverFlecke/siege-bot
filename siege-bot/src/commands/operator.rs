use async_trait::async_trait;
use serenity::{
    builder::{CreateApplicationCommand, CreateEmbed},
    model::{
        prelude::{
            command::CommandOptionType,
            interaction::{
                application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
                autocomplete::AutocompleteInteraction,
                InteractionResponseType,
            },
        },
        Timestamp,
    },
    prelude::Context,
    utils::Color,
};
use siege_api::{models::Statistics, operator::Operator};

use crate::{commands::CommandError, SiegeApi, SiegePlayerLookup};

use super::{get_user_or_default, send_text_message, CommandHandler};

pub struct OperatorCommand;

static NAME: &str = "name";

#[async_trait]
impl CommandHandler for OperatorCommand {
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name("operator")
            .description("Get detailed statistics about an operator")
            .create_option(|option| {
                option
                    .name(NAME)
                    .description("Name of the operator")
                    .kind(CommandOptionType::String)
                    .set_autocomplete(true)
                    .required(true);

                // Note: Left this out for now, as Discord only support up to 25 options 😒

                // use strum::IntoEnumIterator;
                // Operator::iter()
                //     .take(25)
                //     .map(|op| op.to_string())
                //     .for_each(|op| {
                //         option.add_string_choice(op.as_str(), op.as_str());
                //     });

                option
            })
            .create_option(|option| {
                option
                    .name("user")
                    .description("The user to get statistics for. Defaults to the sending user")
                    .kind(CommandOptionType::User)
                    .required(false)
            })
    }

    async fn run(
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), CommandError> {
        let user = get_user_or_default(command);
        let data = ctx.data.read().await;
        let siege_client = data
            .get::<SiegeApi>()
            .expect("Siege client is always registered");

        let operator = if let CommandDataOptionValue::String(value) = command
            .data
            .options
            .iter()
            .find(|x| x.name == NAME)
            .expect("required argument")
            .resolved
            .as_ref()
            .expect("required argument")
        {
            value.parse::<Operator>().unwrap() // TODO: Handle unwrap
        } else {
            todo!()
        };

        tracing::debug!("Getting statistics for operator '{operator}'");

        let lookup = data.get::<SiegePlayerLookup>().expect("always registered");
        let lookup = lookup.read().await;
        match lookup.get(&user.id) {
            Some(player_id) => {
                let response = siege_client.get_operators(*player_id).await.unwrap();

                let operator = match response.get_operator(operator) {
                    Some(operator) => operator,
                    None => {
                        send_text_message(
                            ctx,
                            command,
                            format!("{user} has not played as {operator}", user = user.tag())
                                .as_str(),
                        )
                        .await?;

                        return Ok(());
                    }
                };

                command
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|msg| {
                                msg.embed(|embed| {
                                    embed
                                        .thumbnail(operator.name().avatar_url())
                                        .title(format!(
                                            "Operator statistics for {}",
                                            operator.name()
                                        ))
                                        .color(Color::BLUE).format(operator.statistics())
                                })
                            })
                    })
                    .await
                    .map_err(CommandError::SerenityError)?;
            }
            None => {
                send_text_message(
                    ctx,
                    command,
                    format!("No player found for {}", user.tag()).as_str(),
                )
                .await?
            }
        }

        Ok(())
    }
}

impl OperatorCommand {
    pub async fn handle_autocomplete(
        ctx: &Context,
        interaction: &AutocompleteInteraction,
    ) -> Result<(), CommandError> {
        if let Some(value) = interaction
            .data
            .options
            .iter()
            .find(|option| option.name == NAME)
            .and_then(|x| x.value.clone())
        {
            let value = value.as_str().expect("this should always be a string");
            interaction
                .create_autocomplete_response(&ctx.http, |response| {
                    use strum::IntoEnumIterator;
                    Operator::iter()
                        .map(|op| op.to_string())
                        .filter(|op| op.starts_with(value))
                        .take(25)
                        .for_each(|op| {
                            response.add_string_choice(op.as_str(), op.as_str());
                        });

                    response
                })
                .await
                .map_err(CommandError::SerenityError)?;
        }

        Ok(())
    }
}

trait FormatEmbedded<'a, T> {
    fn format(&'a mut self, value: &T) -> &'a mut Self;
}

/// Create an embedded Discord message with statistics information.
impl FormatEmbedded<'_, Statistics> for CreateEmbed {
    fn format(&mut self, statistics: &Statistics) -> &mut Self {
        let values = vec![
            ("Minutes played", statistics.minutes_played().to_string()),
            (
                "Rounds with a kill",
                format!("{:.2} %", 100f64 * statistics.rounds_with_a_kill()),
            ),
            (
                "Rounds with KOST",
                format!("{:.2} %", 100f64 * statistics.rounds_with_kost()),
            ),
            (
                "Rounds with a multi kills",
                format!("{:.2} %", 100f64 * statistics.rounds_with_multi_kill()),
            ),
            (
                "Rounds with opening kill",
                format!(
                    "{:.2} % ({})",
                    100f64 * statistics.rounds_with_opening_kill(),
                    statistics.opening_kills(),
                ),
            ),
            (
                "Rounds with opening death",
                format!(
                    "{:.2} % ({})",
                    100f64 * statistics.rounds_with_opening_death(),
                    statistics.opening_deaths(),
                ),
            ),
            (
                "Opening winrate",
                format!("{:.2} %", 100f64 * statistics.opening_win_rate()),
            ),
            (
                "Rounds survived",
                format!("{:.2} %", 100f64 * statistics.rounds_survived()),
            ),
            (
                "Rounds with an ace",
                format!("{:.2} %", 100f64 * statistics.rounds_with_an_ace()),
            ),
            (
                "Rounds with a clutch",
                format!("{:.2} %", 100f64 * statistics.rounds_with_clutch()),
            ),
            (
                "Kills per round",
                format!("{:.2} %", 100f64 * statistics.kills_per_round()),
            ),
            (
                "Headshots",
                format!(
                    "{:.2} % ({})",
                    100f64 * statistics.headshot_accuracy(),
                    statistics.headshots(),
                ),
            ),
            ("Melee kills", statistics.melee_kills().to_string()),
            ("Team kills", statistics.team_kills().to_string()),
            ("Trades", statistics.trades().to_string()),
            ("Revives", statistics.revives().to_string()),
            (
                "Time alive per match",
                statistics.time_alive_per_match().to_string(),
            ),
            (
                "Time dead per match",
                statistics.time_dead_per_match().to_string(),
            ),
        ];

        let names = values.iter().map(|x| x.0).collect::<Vec<_>>().join("\n");
        let values = values
            .iter()
            .map(|x| x.1.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        self
            .timestamp(Timestamp::now())
            .field(
                "Kills",
                format!(
                    "K/D: **{kd:.2}** - KDA: **{kills}** / **{deaths}** / **{assists}**",
                    kd = statistics.kill_death_ratio(),
                    kills = statistics.kills(),
                    deaths = statistics.deaths(),
                    assists = statistics.assists(),
                 ),
                false,
            )
            .field(
                "Matches",
                format!(
                    "Win rate **{win_rate:.2} %** Played/Win/Lost: **{total}** / **{wins}** / **{lost}**",
                    win_rate = 100f64 * statistics.matches_win_rate(),
                    total = statistics.matches_played(),
                    wins = statistics.matches_won(),
                    lost = statistics.matches_lost(),
                ),
                false,
            )
            .field(
                "Rounds",
                format!(
                    "Win rate **{win_rate:.2} %** Played/Win/Lost: **{total}** / **{wins}** / **{lost}**",
                    win_rate = 100f64 * statistics.rounds_win_rate(),
                    total = statistics.rounds_played(),
                    wins = statistics.rounds_won(),
                    lost = statistics.rounds_lost(),
                ),
                false,
            )
            .field("Statistic", names, true)
            .field("Value", values, true)
    }
}