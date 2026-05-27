use std::sync::Arc;

use serde::{Deserialize, Serialize};
use telluride::{markdown_format, markdown_string};
use teloxide::prelude::ResponseResult;

use crate::{
    impl_command_execute_2, impl_command_io,
    menus::{select_category::select_category, update_category::update_category},
    storages::{Category, CategoryStorageTrait},
    ui::CommandContext,
};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandRemoveCategory {
    pub category: Option<Category>,
    pub confirm: Option<bool>,
}

impl CommandRemoveCategory {
    async fn run0(
        &self,
        target: &CommandContext,
        storage: Arc<dyn CategoryStorageTrait>,
    ) -> ResponseResult<()> {
        select_category(
            target,
            &storage,
            markdown_string!("✏️ Select Category to remove"),
            |category| {
                CommandRemoveCategory {
                    category: Some(category.clone()),
                    confirm: None,
                }
                .into()
            },
            None,
        )
        .await
    }

    async fn run1(
        &self,
        target: &CommandContext,
        storage: Arc<dyn CategoryStorageTrait>,
        category: &Category,
    ) -> ResponseResult<()> {
        update_category(
            target,
            &storage,
            category,
            markdown_format!("🗑️ Confirm Category `{}` Removal", category.as_str()),
            "🗑️ Remove",
            CommandRemoveCategory {
                category: Some(category.clone()),
                confirm: Some(true),
            }
            .into(),
            Some(
                CommandRemoveCategory {
                    category: None,
                    confirm: None,
                }
                .into(),
            ),
        )
        .await
    }

    async fn run2(
        &self,
        target: &CommandContext,
        storage: Arc<dyn CategoryStorageTrait>,
        category: &Category,
        confirm: &bool,
    ) -> ResponseResult<()> {
        if !*confirm {
            target
                .send_markdown_message(markdown_format!(
                    "❌ Category `{}` removal cancelled\\.",
                    category.as_str()
                ))
                .await?;
            return Ok(());
        }

        if let Err(e) = storage.remove_category(category).await {
            target.send_markdown_message(e).await?;
        }
        target
            .send_markdown_message(markdown_format!(
                "✅ Category `{}` removed\\.",
                category.as_str()
            ))
            .await?;
        Ok(())
    }
}

impl_command_io!(
    CommandRemoveCategory,
    "remove_category",
    ["<name>", "<confirm>"],
    category: Category,
    confirm: bool
);
impl_command_execute_2!(
    CommandRemoveCategory,
    Arc<dyn CategoryStorageTrait>,
    category,
    confirm
);

impl From<CommandRemoveCategory> for crate::commands::Command {
    fn from(cmd: CommandRemoveCategory) -> Self {
        crate::commands::Command::RemoveCategory(cmd)
    }
}
