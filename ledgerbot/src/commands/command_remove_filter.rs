use std::sync::Arc;

use serde::{Deserialize, Serialize};
use telluride::{markdown_format, markdown_string};
use teloxide::prelude::ResponseResult;

use crate::{
    impl_command_execute_3, impl_command_io,
    menus::{
        common::read_category_filter_by_index, select_category::select_category,
        select_category_filter::select_category_filter,
        update_category_filter::update_category_filter,
    },
    storages::{Category, CategoryStorageTrait},
    ui::CommandContext,
};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandRemoveFilter {
    pub category: Option<Category>,
    pub position: Option<usize>,
    pub confirm: Option<bool>,
}

impl CommandRemoveFilter {
    pub fn new(category: Option<Category>, position: Option<usize>) -> Self {
        Self {
            category,
            position,
            confirm: None,
        }
    }

    async fn run0(
        &self,
        target: &CommandContext,
        storage: Arc<dyn CategoryStorageTrait>,
    ) -> ResponseResult<()> {
        select_category(
            target,
            &storage,
            markdown_string!("🗑️ Select Category for removing filter"),
            |category| {
                CommandRemoveFilter {
                    category: Some(category.clone()),
                    position: None,
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
        name: &Category,
    ) -> ResponseResult<()> {
        select_category_filter(
            target,
            &storage,
            name,
            markdown_format!(
                "🗑️ Select Filter to remove from category `{}`",
                name.as_str()
            ),
            |idx, _pattern| {
                Some(
                    CommandRemoveFilter {
                        category: Some(name.clone()),
                        position: Some(idx),
                        confirm: None,
                    }
                    .into(),
                )
            },
            Some(CommandRemoveFilter::default().into()),
        )
        .await
    }

    async fn run2(
        &self,
        target: &CommandContext,
        storage: Arc<dyn CategoryStorageTrait>,
        name: &Category,
        idx: &usize,
    ) -> ResponseResult<()> {
        update_category_filter(
            target,
            &storage,
            name,
            *idx,
            |pattern| {
                markdown_format!(
                    "🗑️ Confirm Filter **\\#{}** \\(`{}`\\) Removal from category `{}`",
                    *idx,
                    pattern,
                    name.as_str()
                )
            },
            "🗑️ Remove",
            |_pattern| {
                CommandRemoveFilter {
                    category: Some(name.clone()),
                    position: Some(*idx),
                    confirm: Some(true),
                }
                .into()
            },
            Some(
                CommandRemoveFilter {
                    category: Some(name.clone()),
                    position: None,
                    confirm: None,
                }
                .into(),
            ),
        )
        .await
    }

    async fn run3(
        &self,
        target: &CommandContext,
        storage: Arc<dyn CategoryStorageTrait>,
        name: &Category,
        idx: &usize,
        confirm: &bool,
    ) -> ResponseResult<()> {
        if !*confirm {
            target
                .send_markdown_message(markdown_format!(
                    "❌ Filter removal from category `{}` cancelled\\.",
                    name.as_str()
                ))
                .await?;
            return Ok(());
        }

        let Some(pattern) = read_category_filter_by_index(
            target,
            &storage,
            name,
            *idx,
            Some(
                CommandRemoveFilter {
                    category: Some(name.clone()),
                    position: None,
                    confirm: None,
                }
                .into(),
            ),
        )
        .await?
        else {
            return Ok(());
        };

        if let Err(e) = storage.remove_category_filter(name, &pattern).await {
            target
                .send_markdown_message(markdown_format!("❌ Failed to remove filter: {}", e))
                .await?;
        }

        target
            .send_markdown_message(markdown_format!(
                "✅ Filter **\\#{}** \\(`{}`\\) removed from category `{}`\\.",
                *idx,
                pattern,
                name.as_str()
            ))
            .await?;

        Ok(())
    }
}

impl_command_io!(
    CommandRemoveFilter,
    "remove_filter",
    ["<category>", "<position>", "<confirm>"],
    category: Category,
    position: usize,
    confirm: bool
);
impl_command_execute_3!(
    CommandRemoveFilter,
    Arc<dyn CategoryStorageTrait>,
    category,
    position,
    confirm
);

impl From<CommandRemoveFilter> for crate::commands::Command {
    fn from(cmd: CommandRemoveFilter) -> Self {
        crate::commands::Command::RemoveFilter(cmd)
    }
}
