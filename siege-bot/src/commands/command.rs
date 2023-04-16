use std::str::FromStr;

use async_trait::async_trait;
use serenity::{
    builder::CreateEmbed,
    http::Http,
    model::{
        prelude::interaction::{
            application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
            InteractionResponseType,
        },
        user::User,
    },
};

use super::{CmdResult, CommandError};

/// Wrapper for a `ApplicationCommandInteraction`.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DiscordAppCmd: Sync + Send {
    /// Extract an option from the command. If not provided, `None` will be
    /// returned.
    fn get_option(&self, name: &str) -> Option<CommandDataOptionValue>;

    /// Extract an option of the given name and parse it into the given type.
    fn extract_enum_option<T>(&self, option_name: &str) -> Option<T>
    where
        T: FromStr + 'static;

    /// Extract the user argument from the command, or if not provide,
    /// return the user who invoked the command.
    fn get_user_from_command_or_default(&self) -> User;

    async fn send_text<H>(&self, http: H, text: &str) -> CmdResult
    where
        H: AsRef<Http> + 'static + Send + Sync;

    async fn send_embedded<H>(&self, http: H, embed: CreateEmbed) -> CmdResult
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

    fn extract_enum_option<T>(&self, option_name: &str) -> Option<T>
    where
        T: FromStr + 'static,
    {
        self.get_option(option_name).and_then(|x| {
            if let CommandDataOptionValue::String(value) = x {
                value.parse::<T>().ok()
            } else {
                None
            }
        })
    }

    fn get_user_from_command_or_default(&self) -> User {
        self.get_option("user")
            .and_then(|x| match x {
                CommandDataOptionValue::User(user, _) => Some(user),
                _ => None,
            })
            .unwrap_or(self.user.clone())
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

    async fn send_embedded<H>(&self, http: H, embed: CreateEmbed) -> CmdResult
    where
        H: AsRef<Http> + 'static + Send + Sync,
    {
        self.create_interaction_response(http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|msg| msg.add_embed(embed))
        })
        .await
        .map_err(CommandError::SerenityError)
    }
}
