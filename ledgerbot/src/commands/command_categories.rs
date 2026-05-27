use std::sync::Arc;

use serde::{Deserialize, Serialize};
use telluride::markdown_format;
use teloxide::prelude::{Requester, ResponseResult};

use crate::{
    commands::{command_add_category::CommandAddCategory, command_add_filter::CommandAddFilter},
    impl_command_execute_0, impl_command_io,
    menus::common::show_follow_status_message,
    storages::{Category, StorageReadonly},
    ui::CommandContext,
};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandCategories;

impl_command_io!(CommandCategories, "categories", []);
impl_command_execute_0!(CommandCategories, Arc<StorageReadonly>);

impl CommandCategories {
    async fn run0(
        &self,
        target: &CommandContext,
        storage: Arc<StorageReadonly>,
    ) -> ResponseResult<()> {
        show_follow_status_message(target, &storage).await?;

        let categories = storage
            .categories_readonly()
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
            let mut sorted_categories: Vec<_> = categories.iter().collect();
            sorted_categories.sort_by(|a, b| a.0.cmp(b.0));

            for (name, patterns) in sorted_categories {
                result.push_str(&CommandAddCategory::new(name).to_command_string(true));
                result.push('\n');

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
            target.bot.send_message(target.chat.id, result).await?;
        }

        Ok(())
    }
}

impl From<CommandCategories> for crate::commands::Command {
    fn from(cmd: CommandCategories) -> Self {
        crate::commands::Command::Categories(cmd)
    }
}
