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

#[cfg(test)]
mod test {
    use serde_json::{json, Value};

    use super::*;

    #[test]
    fn validate_register() {
        let mut command = CreateApplicationCommand::default();
        let command = IdCommand::register(&mut command);

        assert_eq!(command.0.get("name").unwrap(), "id");
        assert_eq!(command.0.get("description").unwrap(), "Get a user id");

        let option = command
            .0
            .get("options")
            .unwrap()
            .as_array()
            .unwrap()
            .get(0)
            .unwrap()
            .as_object()
            .unwrap();

        assert_eq!(option.get("description").unwrap(), "The user to lookup");
        assert_eq!(option.get("name").unwrap(), "id");
        assert_eq!(*option.get("required").unwrap(), Value::Bool(true));
        assert_eq!(*option.get("type").unwrap(), json!(6));
    }
}
