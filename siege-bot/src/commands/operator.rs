use async_trait::async_trait;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandOptionType,
        interaction::{
            application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
            autocomplete::AutocompleteInteraction,
            InteractionResponseType,
        },
    },
    prelude::Context,
    utils::Color,
};
use siege_api::operator::Operator;

use crate::{commands::CommandError, formatting::FormatEmbedded, SiegeApi, SiegePlayerLookup};

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
                    .required(true)
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
            value.parse::<Operator>().unwrap()
        } else {
            unreachable!()
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
                                        .color(Color::BLUE)
                                        .format(operator.statistics())
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
    /// Handle auto complete for operator names.
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
