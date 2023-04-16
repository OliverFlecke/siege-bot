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
    context::DiscordContext, discord_app_command::DiscordAppCmd, AddUserOptionToCommand, CmdResult,
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
                command
                    .send_text(
                        ctx.http(),
                        format!("{user} has not played the '{map:?}' map", user = user.tag())
                            .as_str(),
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

#[cfg(test)]
mod test {
    use serde_json::Value;

    use super::*;

    #[tokio::test]
    async fn validate_register() {
        let mut command = CreateApplicationCommand::default();
        let command = MapCommand::register(&mut command);

        // Assert
        assert_eq!(command.0.get("name").unwrap(), "map");
        assert!(!command
            .0
            .get("description")
            .and_then(|x| x.as_str())
            .unwrap()
            .is_empty());

        let options = command.0.get("options").unwrap().as_array().unwrap();
        // Assert first options
        let opt = options.get(0).unwrap();
        assert_eq!(opt.get("name").unwrap(), NAME);
        assert_eq!(*opt.get("required").unwrap(), Value::Bool(true));
        assert_eq!(opt.get("type").unwrap().as_u64().unwrap(), 3); // Corresponds to `CommandOptionType::User`
        assert_eq!(*opt.get("autocomplete").unwrap(), Value::Bool(true));

        let opt = options.get(1).unwrap();
        assert_eq!(opt.get("name").unwrap(), "user");
        assert_eq!(*opt.get("required").unwrap(), Value::Bool(false));
        assert_eq!(opt.get("type").unwrap().as_u64().unwrap(), 6); // Corresponds to `CommandOptionType::User`
        assert!(!opt.get("description").unwrap().as_str().unwrap().is_empty());
    }
}
