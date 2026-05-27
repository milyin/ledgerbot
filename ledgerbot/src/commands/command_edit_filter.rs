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
pub struct CommandEditFilter {
    pub category: Option<Category>,
    pub position: Option<usize>,
    pub pattern: Option<String>,
}

impl CommandEditFilter {
    async fn run0(
        &self,
        target: &CommandContext,
        storage: Arc<dyn CategoryStorageTrait>,
    ) -> ResponseResult<()> {
        select_category(
            target,
            &storage,
            markdown_string!("✏️ Select Category for editing filter"),
            |category| {
                CommandEditFilter {
                    category: Some(category.clone()),
                    position: None,
                    pattern: None,
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
            markdown_format!("✏️ Select Filter to edit in category `{}`", name.as_str()),
            |idx, _pattern| {
                Some(
                    CommandEditFilter {
                        category: Some(name.clone()),
                        position: Some(idx),
                        pattern: None,
                    }
                    .into(),
                )
            },
            Some(CommandEditFilter::default().into()),
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
                    "✏️ **Editing filter \\#{} in category `{}`:**\n\nCurrent pattern: `{}`",
                    *idx,
                    name.as_str(),
                    pattern
                )
            },
            "✏️ Edit pattern",
            |pattern| {
                CommandEditFilter {
                    category: Some(name.clone()),
                    position: Some(*idx),
                    pattern: Some(pattern.to_string()),
                }
                .into()
            },
            Some(
                CommandEditFilter {
                    category: Some(name.clone()),
                    position: None,
                    pattern: None,
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
        pattern: &String,
    ) -> ResponseResult<()> {
        let Some(old_pattern) = read_category_filter_by_index(
            target,
            &storage,
            name,
            *idx,
            Some(
                CommandEditFilter {
                    category: Some(name.clone()),
                    position: None,
                    pattern: None,
                }
                .into(),
            ),
        )
        .await?
        else {
            return Ok(());
        };

        if let Err(e) = regex::Regex::new(pattern) {
            target
                .send_markdown_message(markdown_format!(
                    "❌ Invalid regex pattern `{}`:\n{}",
                    pattern,
                    &e.to_string()
                ))
                .await?;
            return Ok(());
        }

        if let Err(e) = storage.remove_category_filter(name, &old_pattern).await {
            target
                .send_markdown_message(markdown_format!("❌ Failed to remove filter: {}", e))
                .await?;
        }

        if let Err(e) = storage.add_category_filter(name, pattern.clone()).await {
            target.send_markdown_message(e).await?;
            return Ok(());
        }

        target
            .send_markdown_message(markdown_format!(
                "✅ Filter updated in category `{}`\\.\n`{}` *before*\n`{}` *after*",
                name.as_str(),
                old_pattern.clone(),
                pattern.clone()
            ))
            .await?;

        Ok(())
    }
}

impl_command_io!(
    CommandEditFilter,
    "edit_filter",
    ["<category>", "<position>", "<new_pattern>"],
    category: Category,
    position: usize,
    pattern: String
);
impl_command_execute_3!(
    CommandEditFilter,
    Arc<dyn CategoryStorageTrait>,
    category,
    position,
    pattern
);

impl From<CommandEditFilter> for crate::commands::Command {
    fn from(cmd: CommandEditFilter) -> Self {
        crate::commands::Command::EditFilter(cmd)
    }
}
