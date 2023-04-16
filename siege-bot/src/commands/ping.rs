use async_trait::async_trait;
use serenity::builder::CreateApplicationCommand;

use super::{command::DiscordAppCmd, context::DiscordContext, CmdResult, CommandHandler};

pub struct PingCommand;

#[async_trait]
impl CommandHandler for PingCommand {
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name("ping")
            .description("A ping command to verify that the bot is alive")
    }

    async fn run<Ctx, Cmd>(ctx: &Ctx, command: &Cmd) -> CmdResult
    where
        Ctx: DiscordContext + Send + Sync,
        Cmd: DiscordAppCmd + 'static + Send + Sync,
    {
        ctx.send_text_message(command, "Hey, I'm alive!").await
    }
}
