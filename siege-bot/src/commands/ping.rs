use async_trait::async_trait;
use serenity::builder::CreateApplicationCommand;

use super::{
    context::DiscordContext, discord_app_command::DiscordAppCmd, CmdResult, CommandHandler,
};

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

#[cfg(test)]
mod test {
    use mockall::predicate::*;

    use crate::commands::{context::MockDiscordContext, discord_app_command::MockDiscordAppCmd};

    use super::*;

    #[test]
    fn validate_register() {
        let mut command = CreateApplicationCommand::default();
        let command = PingCommand::register(&mut command);

        assert_eq!(command.0.get("name").unwrap(), "ping");
        assert_eq!(
            command.0.get("description").unwrap(),
            "A ping command to verify that the bot is alive"
        );
    }

    #[tokio::test]
    async fn validate_run() {
        let mut ctx = MockDiscordContext::new();
        ctx.expect_send_text_message::<MockDiscordAppCmd>()
            .once()
            .with(always(), eq("Hey, I'm alive!"))
            .returning(|_, _| Ok(()));
        let cmd = MockDiscordAppCmd::new();

        assert!(PingCommand::run(&ctx, &cmd).await.is_ok());
    }
}
