use async_trait::async_trait;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandOptionType,
        interaction::application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
    },
    prelude::Context,
};

use crate::{commands::send_text_message, SiegeApi};

use super::{get_user_from_command_or_default, CommandError, CommandHandler};

pub struct AddPlayerCommand;

static UBISOFT_NAME: &str = "ubisoft_name";

#[async_trait]
impl CommandHandler for AddPlayerCommand {
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name("add")
            .description("Link your Ubisoft player ID to your Discord Id. This is required before using most commands")
            .create_option(|option| {
                option
                    .name(UBISOFT_NAME)
                    .description("Name used on your Ubisoft account")
                    .kind(CommandOptionType::String)
                    .required(true)
            })
            .create_option(|option| {
                option
                    .name("user")
                    .description("The user to link. Defaults to the sending user")
                    .kind(CommandOptionType::User)
                    .required(false)
            })
    }

    async fn run(
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), CommandError> {
        let name = if let CommandDataOptionValue::String(value) = command
            .data
            .options
            .iter()
            .find(|x| x.name == UBISOFT_NAME)
            .expect("required argument")
            .resolved
            .as_ref()
            .expect("required argument")
        {
            value
        } else {
            unreachable!()
        };

        let user = get_user_from_command_or_default(command);
        tracing::info!(
            "Linking {user} with Ubisoft account {name}",
            user = user.tag()
        );

        let ubisoft_id = {
            let data = ctx.data.read().await;
            let siege_client = data
                .get::<SiegeApi>()
                .expect("Siege client is always registered");
            match siege_client.search_for_player(name).await {
                Ok(id) => id,
                Err(err) => {
                    tracing::error!("Could not find player with that id. Error: {err:?}");
                    return send_text_message(ctx, command, "No player found with that name").await;
                }
            }
        };

        {
            let data = ctx.data.write().await;
            let lookup = data
                .get::<crate::siege_player_lookup::SiegePlayerLookup>()
                .expect("always registered");
            let mut lookup = lookup.write().await;

            match lookup.insert(&user.id, ubisoft_id) {
                Ok(_) => {
                    send_text_message(ctx, command, "Accounts linked!").await?;
                }
                Err(err) => {
                    tracing::error!("Failed to store user: {err:?}");
                    return send_text_message(ctx, command, "Failed to link your accounts").await;
                }
            };
        }

        Ok(())
    }
}
