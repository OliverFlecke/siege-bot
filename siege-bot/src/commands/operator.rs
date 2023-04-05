use async_trait::async_trait;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandOptionType,
        interaction::{
            application_command::ApplicationCommandInteraction,
            autocomplete::AutocompleteInteraction, InteractionResponseType,
        },
    },
    prelude::Context,
    utils::Color,
};
use siege_api::operator::Operator;

use crate::{
    commands::{lookup_siege_player, utils::ExtractEnumOption, CommandError},
    constants::NAME,
    formatting::FormatEmbedded,
    SiegeApi,
};

use super::{
    get_user_from_command_or_default, send_text_message, AddUserOptionToCommand, CommandHandler,
};

pub struct OperatorCommand;

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
            .add_user_option()
    }

    async fn run(
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), CommandError> {
        let operator = command
            .extract_enum_option(NAME)
            .expect("required argument");
        let user = get_user_from_command_or_default(command);
        let player_id = lookup_siege_player(ctx, command, user).await?;

        tracing::info!(
            "Getting statistics for operator '{operator}' for {}",
            user.name
        );

        let response = {
            let data = ctx.data.read().await;
            let siege_client = data
                .get::<SiegeApi>()
                .expect("Siege client is always registered");
            siege_client.get_operators(player_id).await.unwrap()
        };

        let operator = match response.get_operator(operator) {
            Some(operator) => operator,
            None => {
                send_text_message(
                    ctx,
                    command,
                    format!("{user} has not played as {operator}", user = user.tag()).as_str(),
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
                                .title(format!("Operator statistics for {}", operator.name()))
                                .color(Color::BLUE)
                                .format(operator.statistics())
                        })
                    })
            })
            .await
            .map_err(CommandError::SerenityError)?;

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
