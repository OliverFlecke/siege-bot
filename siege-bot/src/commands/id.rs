use async_trait::async_trait;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandOptionType,
        interaction::application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
    },
    prelude::Context,
};

use super::{send_text_message, CommandHandler};

pub struct IdCommand;

#[async_trait]
impl CommandHandler for IdCommand {
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name("id")
            .description("Get a user id")
            .create_option(|option| {
                option
                    .name("id")
                    .description("The user to lookup")
                    .kind(CommandOptionType::User)
                    .required(true)
            })
    }

    async fn run(
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), super::CommandError> {
        let option = command
            .data
            .options
            .get(0)
            .expect("Expected user option")
            .resolved
            .as_ref()
            .expect("Expected user object");

        let content = if let CommandDataOptionValue::User(user, _member) = option {
            format!("{}'s id is {}", user.tag(), user.id)
        } else {
            "Please provide a valid user".to_string()
        };

        send_text_message(ctx, command, content.as_str()).await?;

        Ok(())
    }
}
