use std::sync::Arc;

use teloxide::prelude::{Requester, ResponseResult};
use yoroolbot::markdown_format;

use crate::{
    commands::{
        Command,
        command_add_category::CommandAddCategory,
        command_trait::{CommandReplyTarget, CommandTrait, EmptyArg},
    },
    storage_traits::CategoryStorageTrait,
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandCategories;

impl CommandTrait for CommandCategories {
    type A = EmptyArg;
    type B = EmptyArg;
    type C = EmptyArg;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<dyn CategoryStorageTrait>;

    const NAME: &'static str = "categories";
    const PLACEHOLDERS: &[&'static str] = &[];

    fn from_arguments(
        _: Option<Self::A>,
        _: Option<Self::B>,
        _: Option<Self::C>,
        _: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandCategories
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
    ) -> ResponseResult<()> {
        let chat_id = target.chat.id;
        let categories = storage.get_chat_categories(chat_id).await;

        if categories.is_empty() {
            target
                .send_markdown_message(markdown_format!(
                    "📂 No categories defined yet\\. Use {} to create one\\.",
                    CommandAddCategory::default().to_command_string(true)
                ))
                .await?;
        } else {
            let mut result = String::new();

            // Sort categories for consistent output
            let mut sorted_categories: Vec<_> = categories.iter().collect();
            sorted_categories.sort_by(|a, b| a.0.cmp(b.0));

            for (name, patterns) in sorted_categories {
                // First create the category
                result.push_str(&CommandAddCategory::new(name).to_command_string(true));
                result.push('\n');

                // Then assign patterns if they exist
                for pattern in patterns {
                    result.push_str(
                        &Command::AddFilter {
                            category: Some(name.clone()),
                            pattern: Some(pattern.clone()),
                        }
                        .to_string(),
                    );
                    result.push('\n');
                }
            }
            target.bot.send_message(chat_id, result).await?;
        }

        Ok(())
    }
}

impl From<CommandCategories> for crate::commands::Command {
    fn from(cmd: CommandCategories) -> Self {
        crate::commands::Command::Categories(cmd)
    }
}
