use async_trait::async_trait;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::interaction::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
    prelude::Context,
};
use thiserror::Error;

pub mod id;
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
    #[error("intenal Discord error")]
    SerenityError(serenity::Error),
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
