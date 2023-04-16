use async_trait::async_trait;
use serenity::{
    builder::{CreateApplicationCommand, CreateEmbed},
    model::prelude::{
        command::CommandOptionType, interaction::autocomplete::AutocompleteInteraction,
    },
    prelude::Context,
    utils::Color,
};
use siege_api::maps::Map;

use crate::{constants::NAME, formatting::FormatEmbedded, SiegeApi};

use super::{
    command::DiscordAppCmd, context::DiscordContext, AddUserOptionToCommand, CmdResult,
    CommandError, CommandHandler,
};

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

    async fn run<Ctx, Cmd>(ctx: &Ctx, command: &Cmd) -> CmdResult
    where
        Ctx: DiscordContext + Send + Sync,
        Cmd: DiscordAppCmd + 'static + Send + Sync,
    {
        let map = command
            .extract_enum_option(NAME)
            .expect("required argument");

        let user = command.get_user_from_command_or_default();
        let player_id = ctx.lookup_siege_player(command, &user).await?;

        tracing::info!("Getting statistics for map '{map:?}' for {}", user.name);

        let response = {
            let data = ctx.data().read().await;
            let siege_client = data
                .get::<SiegeApi>()
                .expect("Siege client is always registered");
            siege_client.get_maps(player_id).await.unwrap()
        };

        let map = match response.get_map(map) {
            Some(map) => map,
            None => {
                ctx.send_text_message(
                    command,
                    format!("{user} has not played the '{map:?}' map", user = user.tag()).as_str(),
                )
                .await?;

                return Ok(());
            }
        };

        command
            .send_embedded(
                ctx.http().clone(),
                CreateEmbed::default()
                    .thumbnail(map.name().image())
                    .title(format!("Map statistics for {:?}", map.name()))
                    .color(Color::GOLD)
                    .format(map.statistics())
                    .clone(),
            )
            .await
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
