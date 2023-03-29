use async_trait::async_trait;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::interaction::application_command::ApplicationCommandInteraction,
    prelude::Context,
};

use super::{send_text_message, CommandHandler};

pub struct PingCommand;

#[async_trait]
impl CommandHandler for PingCommand {
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name("ping")
            .description("A ping command to verify that the bot is alive")
    }

    async fn run(
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), super::CommandError> {
        send_text_message(ctx, command, "Hey, I'm alive!").await
    }
}
