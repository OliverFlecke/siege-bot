use async_trait::async_trait;
use serenity::{builder::CreateApplicationCommand, model::prelude::command::CommandOptionType};
use thiserror::Error;

use self::{context::DiscordContext, discord_app_command::DiscordAppCmd};

pub mod add_player;
pub mod all_maps;
pub mod all_operators;
pub mod context;
pub mod discord_app_command;
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
