use super::{CmdResult, CommandError};
use crate::constants::{NAME, USER};
use async_trait::async_trait;
use serenity::{
    all::UserId,
    builder::{
        CreateAutocompleteResponse, CreateEmbed, CreateInteractionResponse,
        CreateInteractionResponseMessage,
    },
    http::Http,
    model::prelude::{CommandDataOptionValue, CommandInteraction},
};
use std::{str::FromStr, sync::Arc};

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
    fn get_user_from_command_or_default(&self) -> UserId;

    async fn send_text(&self, http: Option<Arc<Http>>, text: &str) -> CmdResult;

    async fn send_embedded(&self, http: Option<Arc<Http>>, embed: CreateEmbed) -> CmdResult;
}

/// Implementation for wrapper trait. Is mostly transparent + a utility methods
/// to extract data.
#[async_trait]
impl DiscordAppCmd for CommandInteraction {
    fn get_option(&self, name: &str) -> Option<CommandDataOptionValue> {
        self.data
            .options
            .iter()
            .find(|x| x.name == name)
            .map(|x| x.value)
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

    fn get_user_from_command_or_default(&self) -> UserId {
        self.get_option(USER)
            .and_then(|x| match x {
                CommandDataOptionValue::User(user_id) => Some(user_id),
                _ => None,
            })
            .unwrap_or(self.user.id.clone())
    }

    /// Send a basic text message to the current channel.
    async fn send_text(&self, http: Option<Arc<Http>>, text: &str) -> CmdResult {
        self.create_response(
            http.expect("http should always be set when sending embedded"),
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::default().content(text),
            ),
        )
        .await
        .map_err(CommandError::SerenityError)
    }

    /// Send an embedded message to the given channel.
    async fn send_embedded(&self, http: Option<Arc<Http>>, embed: CreateEmbed) -> CmdResult {
        self.create_response(
            http.expect("http should always be set when sending embedded"),
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::default().add_embed(embed),
            ),
        )
        .await
        .map_err(CommandError::SerenityError)
    }
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DiscordAutocompleteInteraction {
    fn get_user_input(&self) -> Option<String>;

    async fn create_autocomplete_response<F>(&self, http: Option<Arc<Http>>, f: F) -> CmdResult
    where
        F: FnOnce(&mut CreateAutocompleteResponse) -> &mut CreateAutocompleteResponse
            + 'static
            + Send
            + Sync;
}

#[async_trait]
impl DiscordAutocompleteInteraction for AutocompleteInteraction {
    fn get_user_input(&self) -> Option<String> {
        self.data
            .options
            .iter()
            .find(|option| option.name == NAME)
            .and_then(|x| x.value.clone())
            .and_then(|x| x.as_str().map(|s| s.to_string()))
    }

    async fn create_autocomplete_response<F>(&self, http: Option<Arc<Http>>, f: F) -> CmdResult
    where
        F: FnOnce(&mut CreateAutocompleteResponse) -> &mut CreateAutocompleteResponse
            + 'static
            + Send
            + Sync,
    {
        self.create_autocomplete_response(http.expect("http always ok for autocompletion"), f)
            .await
            .map_err(CommandError::SerenityError)
    }
}
