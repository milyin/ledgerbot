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
pub struct CommandRenameCategory {
    pub old_category: Option<Category>,
    pub new_category: Option<Category>,
}

impl CommandRenameCategory {
    async fn run0(
        &self,
        target: &CommandContext,
        storage: Arc<dyn CategoryStorageTrait>,
    ) -> ResponseResult<()> {
        select_category(
            target,
            &storage,
            markdown_string!("✏️ Select Category to rename"),
            |category| {
                CommandRenameCategory {
                    old_category: Some(category.clone()),
                    new_category: None,
                }
                .into()
            },
            None,
        )
        .await?;

        Ok(())
    }

    async fn run1(
        &self,
        target: &CommandContext,
        storage: Arc<dyn CategoryStorageTrait>,
        old_category: &Category,
    ) -> ResponseResult<()> {
        update_category(
            target,
            &storage,
            old_category,
            markdown_format!("✏️ Renaming category `{}`", old_category.as_str()),
            "✏️ Rename",
            CommandRenameCategory {
                old_category: Some(old_category.clone()),
                new_category: None,
            }
            .into(),
            Some(
                CommandRenameCategory {
                    old_category: None,
                    new_category: None,
                }
                .into(),
            ),
        )
        .await?;

        Ok(())
    }

    async fn run2(
        &self,
        target: &CommandContext,
        storage: Arc<dyn CategoryStorageTrait>,
        old_category: &Category,
        new_category: &Category,
    ) -> ResponseResult<()> {
        if let Err(e) = storage.rename_category(old_category, new_category).await {
            target.send_markdown_message(e).await?;
        }
        target
            .send_markdown_message(markdown_format!(
                "✅ Category `{}` renamed to `{}`\\.",
                old_category.as_str(),
                new_category.as_str()
            ))
            .await?;
        Ok(())
    }
}

impl_command_io!(
    CommandRenameCategory,
    "rename_category",
    ["<old_name>", "<new_name>"],
    old_category: Category,
    new_category: Category
);
impl_command_execute_2!(
    CommandRenameCategory,
    Arc<dyn CategoryStorageTrait>,
    old_category,
    new_category
);

impl From<CommandRenameCategory> for crate::commands::Command {
    fn from(cmd: CommandRenameCategory) -> Self {
        crate::commands::Command::RenameCategory(cmd)
    }
}
