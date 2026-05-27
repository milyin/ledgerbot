use std::sync::Arc;

use serde::{Deserialize, Serialize};
use telluride::markdown_format;
use teloxide::prelude::ResponseResult;

use crate::{
    commands::command_add_words_filter::CommandAddWordsFilter,
    impl_command_execute_2, impl_command_io,
    storages::{Category, Storage},
    ui::CommandContext,
};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandAddFilter {
    pub category: Option<Category>,
    pub pattern: Option<String>,
}

impl CommandAddFilter {
    async fn run0(&self, target: &CommandContext, storage: Arc<Storage>) -> ResponseResult<()> {
        CommandAddWordsFilter::default()
            .execute(target, storage)
            .await
    }

    async fn run1(
        &self,
        target: &CommandContext,
        storage: Arc<Storage>,
        category: &Category,
    ) -> ResponseResult<()> {
        CommandAddWordsFilter {
            category: Some(category.clone()),
            page: None,
            words: None,
        }
        .execute(target, storage)
        .await
    }

    async fn run2(
        &self,
        target: &CommandContext,
        storage: Arc<Storage>,
        category: &Category,
        pattern: &String,
    ) -> ResponseResult<()> {
        if let Err(msg) = storage
            .categories()
            .add_category_filter(category, pattern.clone())
            .await
        {
            target.send_markdown_message(msg).await?;
            return Ok(());
        };
        target
            .send_markdown_message(markdown_format!(
                "✅ Filter `{}` added to category `{}`\\.",
                pattern,
                category.as_str()
            ))
            .await?;
        Ok(())
    }
}

impl_command_io!(
    CommandAddFilter,
    "add_filter",
    ["<category>", "<pattern>"],
    category: Category,
    pattern: String
);
impl_command_execute_2!(CommandAddFilter, Arc<Storage>, category, pattern);

impl From<CommandAddFilter> for crate::commands::Command {
    fn from(cmd: CommandAddFilter) -> Self {
        crate::commands::Command::AddFilter(cmd)
    }
}
