use async_trait::async_trait;
use serenity::{
    builder::CreateApplicationCommand,
    model::{
        prelude::interaction::{
            application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
            InteractionResponseType,
        },
        user::User,
    },
    prelude::Context,
};
use thiserror::Error;
use uuid::Uuid;

pub mod add_player;
pub mod id;
pub mod map;
pub mod operator;
pub mod ping;
pub mod statistics;

#[async_trait]
pub trait CommandHandler {
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand;

    async fn run(
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), CommandError>;
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("command does not exists")]
    CommandNotFound,
    #[error("internal Discord error")]
    SerenityError(serenity::Error),
    #[error("Siege player not found")]
    SiegePlayerNotFound,
}

/// Utility method to send text back to the channel.
async fn send_text_message(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    content: &str,
) -> Result<(), CommandError> {
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(content))
        })
        .await
        .map_err(CommandError::SerenityError)
}

/// Extract the user argument from the command, or if not provide, return the user who invoked the command.
fn get_user_from_command_or_default(command: &ApplicationCommandInteraction) -> &User {
    match command
        .data
        .options
        .iter()
        .filter(|x| x.name == "user")
        .last()
    {
        Some(opt) => match opt.resolved.as_ref() {
            Some(CommandDataOptionValue::User(user, _)) => user,
            _ => &command.user,
        },
        _ => &command.user,
    }
}

/// Retreive the Siege Id for a Discord user. If it is not found, an error is sent back through the command
/// and an error is returned.
async fn lookup_siege_player(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    user: &User,
) -> Result<Uuid, CommandError> {
    let data = ctx.data.read().await;
    let lookup = data
        .get::<crate::siege_player_lookup::SiegePlayerLookup>()
        .expect("always registered");
    let lookup = lookup.read().await;

    match lookup.get(&user.id) {
        Some(player_id) => Ok(*player_id),
        None => {
            send_text_message(
                ctx,
                command,
                format!(
                    "No Siege player found for {}.\nUse the `/add` command to link your Discord profile to your Ubisoft name",
                    user.tag()
                ).as_str(),
            )
            .await?;
            Err(CommandError::SiegePlayerNotFound)
        }
    }
}
