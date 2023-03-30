use async_trait::async_trait;
use serenity::{
    builder::{CreateApplicationCommand, CreateEmbed},
    model::{
        prelude::{
            command::CommandOptionType,
            interaction::{
                application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
                InteractionResponseType,
            },
        },
        Timestamp,
    },
    prelude::Context,
    utils::Color,
};
use siege_api::models::Operator;

use crate::{commands::CommandError, SiegeApi, SiegePlayerLookup};

use super::{get_user_or_default, send_text_message, CommandHandler};

pub struct OperatorCommand;

#[async_trait]
impl CommandHandler for OperatorCommand {
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name("operator")
            .description("Get detailed statistics about an operator")
            .create_option(|option| {
                option
                    .name("operator")
                    .description("Name of the operator")
                    .kind(CommandOptionType::String)
                    .required(true)
                    .add_string_choice("Aruni", "Aruni")
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

        let operator_name = if let CommandDataOptionValue::String(value) = command
            .data
            .options
            .iter()
            .find(|x| x.name == "operator")
            .expect("required argument")
            .resolved
            .as_ref()
            .expect("required argument")
        {
            value
        } else {
            todo!()
        };

        let lookup = data.get::<SiegePlayerLookup>().expect("always registered");
        let lookup = lookup.read().await;
        match lookup.get(&user.id) {
            Some(player_id) => {
                let operators = siege_client.get_operators(*player_id).await.unwrap();

                let operator = match operators.get_operator(operator_name.as_str()) {
                    Some(operator) => operator,
                    None => todo!(),
                };

                command
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|msg| {
                                msg.embed(|e| {
                                    create_embedded_operator(operator, e).color(Color::BLUE)
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

/// Create an embedded Discord message for an operator
fn create_embedded_operator<'a>(
    operator: &Operator,
    e: &'a mut CreateEmbed,
) -> &'a mut CreateEmbed {
    let values = vec![
        (
            "Rounds with a kill",
            format!("{:.2} %", 100f64 * operator.rounds_with_a_kill()),
        ),
        (
            "Rounds with KOST",
            format!("{:.2} %", 100f64 * operator.rounds_with_kost()),
        ),
        (
            "Rounds with a multi kills",
            format!("{:.2} %", 100f64 * operator.rounds_with_multi_kill()),
        ),
        (
            "Rounds with opening kill",
            format!(
                "{:.2} % ({})",
                100f64 * operator.rounds_with_opening_kill(),
                operator.opening_kills(),
            ),
        ),
        (
            "Rounds with opening death",
            format!(
                "{:.2} % ({})",
                100f64 * operator.rounds_with_opening_death(),
                operator.opening_deaths(),
            ),
        ),
        (
            "Opening winrate",
            format!("{:.2} %", 100f64 * operator.opening_win_rate()),
        ),
        (
            "Rounds survived",
            format!("{:.2} %", 100f64 * operator.rounds_survived()),
        ),
        (
            "Rounds with an ace",
            format!("{:.2} %", 100f64 * operator.rounds_with_an_ace()),
        ),
        (
            "Rounds with a clutch",
            format!("{:.2} %", 100f64 * operator.rounds_with_clutch()),
        ),
        (
            "Kills per round",
            format!("{:.2} %", 100f64 * operator.kills_per_round()),
        ),
        (
            "Headshots",
            format!(
                "{:.2} % ({})",
                100f64 * operator.headshot_accuracy(),
                operator.headshots(),
            ),
        ),
        ("Melee kills", operator.melee_kills().to_string()),
        ("Team kills", operator.team_kills().to_string()),
        ("Trades", operator.trades().to_string()),
        ("Revives", operator.revives().to_string()),
        (
            "Time alive per match",
            operator.time_alive_per_match().to_string(),
        ),
        (
            "Time dead per match",
            operator.time_dead_per_match().to_string(),
        ),
    ];

    let names = values.iter().map(|x| x.0).collect::<Vec<_>>().join("\n");
    let values = values
        .iter()
        .map(|x| x.1.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    e.title(format!("Operator statistics for {}", operator.name()))
        .timestamp(Timestamp::now())
        .thumbnail(operator.avatar_url())
        .field(
            "Kills", 
            format!(
                "K/D: **{kd:.2}** - KDA: **{kills}** / **{deaths}** / **{assists}**", 
                kd = operator.kill_death_ratio(),
                kills = operator.kills(),
                deaths = operator.deaths(),
                assists = operator.assists(),
             ),
            false,
        )
        .field(
            "Matches",
            format!(
                "Win rate **{win_rate:.2} %** Played/Win/Lost: **{total}** / **{wins}** / **{lost}**",
                win_rate = 100f64 * operator.matches_win_rate(),
                total = operator.matches_played(),
                wins = operator.matches_won(),
                lost = operator.matches_lost(),
            ),
            false,
        )
        .field(
            "Rounds",
            format!(
                "Win rate **{win_rate:.2} %** Played/Win/Lost: **{total}** / **{wins}** / **{lost}**",
                win_rate = 100f64 * operator.rounds_win_rate(),
                total = operator.rounds_played(),
                wins = operator.rounds_won(),
                lost = operator.rounds_lost(),
            ),
            false,
        )
        .field("Statistic", names, true)
        .field("Value", values, true)
}
