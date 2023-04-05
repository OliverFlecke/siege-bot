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
    prelude::{self, Context},
    utils::Color,
};
use siege_api::maps::Map;

use crate::{
    commands::{get_user_from_command_or_default, lookup_siege_player, send_text_message},
    constants::NAME,
    formatting::FormatEmbedded,
    SiegeApi,
};

use super::{AddUserOptionToCommand, CommandError, CommandHandler};

pub struct MapCommand;

#[async_trait]
impl CommandHandler for MapCommand {
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name("map")
            .description("Get detailed statistics about a specific map")
            .create_option(|option| {
                option
                    .name(NAME)
                    .description("Name of the map")
                    .kind(CommandOptionType::String)
                    .set_autocomplete(true)
                    .required(true)
            })
            .add_user_option()
    }

    async fn run(
        ctx: &prelude::Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), CommandError> {
        let map = if let CommandDataOptionValue::String(value) = command
            .data
            .options
            .iter()
            .find(|x| x.name == NAME)
            .expect("required argument")
            .resolved
            .as_ref()
            .expect("required argument")
        {
            value.parse::<Map>().expect("should always be valid")
        } else {
            unreachable!()
        };

        tracing::debug!("Getting statistics for map '{map:?}'");

        let user = get_user_from_command_or_default(command);
        let player_id = lookup_siege_player(ctx, command, user).await?;

        let response = {
            let data = ctx.data.read().await;
            let siege_client = data
                .get::<SiegeApi>()
                .expect("Siege client is always registered");
            siege_client.get_maps(player_id).await.unwrap()
        };

        let map = match response.get_map(map) {
            Some(map) => map,
            None => {
                send_text_message(
                    ctx,
                    command,
                    format!("{user} has not played the '{map:?}' map", user = user.tag()).as_str(),
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
                                .thumbnail(map.name().image())
                                .title(format!("Map statistics for {:?}", map.name()))
                                .color(Color::GOLD)
                                .format(map.statistics())
                        })
                    })
            })
            .await
            .map_err(CommandError::SerenityError)?;

        Ok(())
    }
}

impl MapCommand {
    /// Handle auto complete for map names.
    pub async fn handle_autocomplete(
        ctx: &Context,
        interaction: &AutocompleteInteraction,
    ) -> Result<(), CommandError> {
        if let Some(value) = interaction
            .data
            .options
            .iter()
            .find(|option| option.name == NAME)
            .and_then(|x| x.value.to_owned())
        {
            let value = value.as_str().expect("this should always be a string");

            interaction
                .create_autocomplete_response(&ctx.http, |response| {
                    use strum::IntoEnumIterator;

                    Map::iter()
                        .map(|map| map.to_string().replace(' ', ""))
                        .filter(|map| map.starts_with(value))
                        .take(25)
                        .for_each(|map| {
                            response.add_string_choice(map.as_str(), map.as_str());
                        });

                    response
                })
                .await
                .map_err(CommandError::SerenityError)?;
        }

        Ok(())
    }
}
