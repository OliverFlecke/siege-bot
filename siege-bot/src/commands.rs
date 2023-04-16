use async_trait::async_trait;
use serenity::{builder::CreateApplicationCommand, model::prelude::command::CommandOptionType};
use thiserror::Error;

use self::{command::DiscordAppCmd, context::DiscordContext};

pub mod add_player;
pub mod all_maps;
pub mod all_operators;
pub mod command;
pub mod context;
pub mod id;
pub mod map;
pub mod operator;
pub mod ping;
pub mod statistics;

#[async_trait]
pub trait CommandHandler {
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand;

    async fn run<Ctx, Cmd>(ctx: &Ctx, command: &Cmd) -> CmdResult
    where
        Ctx: DiscordContext + Send + Sync,
        Cmd: DiscordAppCmd + 'static + Send + Sync;
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

pub type CmdResult = core::result::Result<(), CommandError>;

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
