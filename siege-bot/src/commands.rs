use async_trait::async_trait;
use serenity::{
    builder::CreateApplicationCommand,
    model::{
        prelude::{
            command::CommandOptionType,
            interaction::{
                application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
                InteractionResponseType,
            },
        },
        user::User,
    },
    prelude::Context,
};
use thiserror::Error;
use uuid::Uuid;

pub mod add_player;
pub mod all_operators;
pub mod id;
pub mod map;
pub mod operator;
pub mod ping;
pub mod statistics;
pub mod all_maps;

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
    command
        .data
        .options
        .iter()
        .filter(|x| x.name == "user")
        .last()
        .and_then(|x| match x.resolved.as_ref() {
            Some(CommandDataOptionValue::User(user, _)) => Some(user),
            _ => None,
        })
        .unwrap_or(&command.user)
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

// TODO: Should these traits/functions be moved to a utils module?

trait AddUserOptionToCommand {
    fn add_user_option(&mut self) -> &mut Self;
}

impl AddUserOptionToCommand for CreateApplicationCommand {
    fn add_user_option(&mut self) -> &mut CreateApplicationCommand {
        self.create_option(|option| {
            option
                .name("user")
                .description("The user to get statistics for. Defaults to the sending user")
                .kind(CommandOptionType::User)
                .required(false)
        })
    }
}

mod utils {
    use std::str::FromStr;

    use serenity::model::prelude::interaction::application_command::{
        ApplicationCommandInteraction, CommandDataOptionValue,
    };

    pub trait ExtractEnumOption {
        fn extract_enum_option<T>(&self, option_name: &str) -> Option<T>
        where
            T: FromStr;
    }

    impl ExtractEnumOption for &ApplicationCommandInteraction {
        fn extract_enum_option<T>(&self, option_name: &str) -> Option<T>
        where
            T: FromStr,
        {
            self.data
                .options
                .iter()
                .find(|x| x.name == option_name)
                .and_then(|x| x.resolved.as_ref())
                .and_then(|x| {
                    if let CommandDataOptionValue::String(value) = x {
                        value.parse::<T>().ok()
                    } else {
                        None
                    }
                })
        }
    }
}
