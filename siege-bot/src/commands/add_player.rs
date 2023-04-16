use async_trait::async_trait;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandOptionType, interaction::application_command::CommandDataOptionValue,
    },
};

use crate::SiegeApi;

use super::{command::DiscordAppCmd, context::DiscordContext, CmdResult, CommandHandler};

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

    async fn run<Ctx, Cmd>(ctx: &Ctx, command: &Cmd) -> CmdResult
    where
        Ctx: DiscordContext + Send + Sync,
        Cmd: DiscordAppCmd + 'static + Send + Sync,
    {
        let name =
            if let Some(CommandDataOptionValue::String(value)) = command.get_option(UBISOFT_NAME) {
                value
            } else {
                unreachable!()
            };

        let user = command.get_user_from_command_or_default();
        tracing::info!(
            "Linking {user} with Ubisoft account {name}",
            user = user.tag()
        );

        let ubisoft_id = {
            let data = ctx.data().read().await;
            let siege_client = data
                .get::<SiegeApi>()
                .expect("Siege client is always registered");
            match siege_client.search_for_player(&name).await {
                Ok(id) => id,
                Err(err) => {
                    tracing::error!("Could not find player with that id. Error: {err:?}");
                    return ctx
                        .send_text_message(command, "No player found with that name")
                        .await;
                }
            }
        };

        {
            let data = ctx.data().write().await;
            let lookup = data
                .get::<crate::siege_player_lookup::SiegePlayerLookup>()
                .expect("always registered");
            let mut lookup = lookup.write().await;

            match lookup.insert(&user.id, ubisoft_id) {
                Ok(_) => {
                    ctx.send_text_message(command, "Accounts linked!").await?;
                }
                Err(err) => {
                    tracing::error!("Failed to store user: {err:?}");
                    return ctx
                        .send_text_message(command, "Failed to link your accounts")
                        .await;
                }
            };
        }

        Ok(())
    }
}
