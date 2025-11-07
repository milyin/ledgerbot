use std::sync::Arc;

use serde::{Deserialize, Serialize};
use teloxide::prelude::{Requester, ResponseResult};
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg},
    markdown_format,
};

use crate::{
    commands::{
        command_add_category::CommandAddCategory, command_add_filter::CommandAddFilter,
        follow_helper::validate_and_get_follow_access,
    },
    storages::{Category, Stores},
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
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

    type Context = Arc<Stores>;

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
        // Validate follow access and get effective chat ID
        let follow_access = match validate_and_get_follow_access(target, &storage).await {
            Ok(access) => access,
            Err(warning_msg) => {
                target.send_markdown_message(warning_msg).await?;
                // Continue with current chat
                crate::commands::follow_helper::FollowAccess {
                    effective_chat_id: target.chat.id,
                    header_note: None,
                }
            }
        };

        let chat_id = follow_access.effective_chat_id;

        // Show follow status as separate message if applicable
        if let Some(header) = follow_access.header_note {
            target.send_markdown_message(header).await?;
        }

        let storage_ = storage.storage(chat_id);
        let categories = storage_
            .categories()
            .get_categories()
            .await
            .unwrap_or_default();

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
                        CommandAddFilter {
                            category: Category::from_string(name).ok(),
                            pattern: Some(pattern.clone()),
                        }
                        .to_command_string(true)
                        .as_str(),
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
