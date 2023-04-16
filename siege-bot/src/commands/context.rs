use async_trait::async_trait;
use serenity::{
    http::Http,
    model::prelude::interaction::{
        application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
        InteractionResponseType,
    },
    prelude::Context,
};

use super::{CmdResult, CommandError};

/// Wrapper for the `serenity::Context` for mocking.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DiscordContext {
    async fn send_text_message<C>(&self, command: &C, content: &str) -> CmdResult
    where
        C: DiscordAppCmd + 'static;
}

/// Wrapper for a `ApplicationCommandInteraction`.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DiscordAppCmd: Sync + Send {
    fn get_option(&self, name: &str) -> Option<CommandDataOptionValue>;

    async fn send_text<H>(&self, http: H, text: &str) -> CmdResult
    where
        H: AsRef<Http> + 'static + Send + Sync;
}

/// Implementation for wrapper trait. Is mostly transparent + a utility methods
/// to extract data.
#[async_trait]
impl DiscordAppCmd for ApplicationCommandInteraction {
    fn get_option(&self, name: &str) -> Option<CommandDataOptionValue> {
        self.data
            .options
            .iter()
            .find(|x| x.name == name)
            .and_then(|x| x.resolved.as_ref())
            .cloned()
    }

    async fn send_text<H>(&self, http: H, text: &str) -> CmdResult
    where
        H: AsRef<Http> + 'static + Send + Sync,
    {
        self.create_interaction_response(http.as_ref(), |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(text))
        })
        .await
        .map_err(CommandError::SerenityError)
    }
}

/// Implementation for wrapper trait.
#[async_trait]
impl DiscordContext for Context {
    async fn send_text_message<C>(&self, command: &C, content: &str) -> CmdResult
    where
        C: DiscordAppCmd,
    {
        command.send_text(self.http.clone(), content).await
    }
}
