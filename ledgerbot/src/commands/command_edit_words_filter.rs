use std::sync::Arc;

use serde::{Deserialize, Serialize};
use telluride::{markdown_format, markdown_string};
use teloxide::prelude::ResponseResult;

use crate::{
    commands::command_edit_filter::CommandEditFilter,
    impl_command_execute_4, impl_command_io,
    menus::{
        common::read_category_filter_by_index,
        select_category::select_category,
        select_category_filter::select_category_filter,
        select_word::{Words, select_word},
    },
    storages::{Category, Stores},
    ui::CommandContext,
    utils::extract_words::extract_and_merge_words,
};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandEditWordsFilter {
    pub category: Option<Category>,
    pub position: Option<usize>,
    pub page: Option<usize>,
    pub words: Option<Words>,
}

impl CommandEditWordsFilter {
    async fn run0(&self, target: &CommandContext, storage: Arc<Stores>) -> ResponseResult<()> {
        let storage_ = storage.storage(target.chat.id);
        select_category(
            target,
            &storage_.categories(),
            markdown_string!("✏️ Select Category to edit word filter"),
            |category| {
                CommandEditWordsFilter {
                    category: Some(category.clone()),
                    position: None,
                    page: None,
                    words: None,
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
        storage: Arc<Stores>,
        category: &Category,
    ) -> ResponseResult<()> {
        let storage_ = storage.storage(target.chat.id);
        select_category_filter(
            target,
            &storage_.categories(),
            category,
            markdown_format!(
                "✏️ Select word\\-based filter to edit in category `{}`",
                category.as_str()
            ),
            |idx, pattern| {
                Words::read_pattern(pattern).map(|_| {
                    CommandEditWordsFilter {
                        category: Some(category.clone()),
                        position: Some(idx),
                        page: None,
                        words: None,
                    }
                    .into()
                })
            },
            Some(CommandEditWordsFilter::default().into()),
        )
        .await
    }

    async fn run2(
        &self,
        target: &CommandContext,
        storage: Arc<Stores>,
        category: &Category,
        position: &usize,
    ) -> ResponseResult<()> {
        let storage_ = storage.storage(target.chat.id);
        let Some(current_pattern) = read_category_filter_by_index(
            target,
            &storage_.categories(),
            category,
            *position,
            Some(
                CommandEditWordsFilter {
                    category: Some(category.clone()),
                    position: None,
                    page: None,
                    words: None,
                }
                .into(),
            ),
        )
        .await?
        else {
            return Ok(());
        };

        let words = Words::read_pattern(&current_pattern).unwrap_or_default();
        self.run4(target, storage, category, position, &0, &words)
            .await
    }

    async fn run3(
        &self,
        target: &CommandContext,
        storage: Arc<Stores>,
        category: &Category,
        position: &usize,
        page: &usize,
    ) -> ResponseResult<()> {
        self.run4(target, storage, category, position, page, &Words::default())
            .await
    }

    async fn run4(
        &self,
        target: &CommandContext,
        storage: Arc<Stores>,
        category: &Category,
        position: &usize,
        page: &usize,
        selected_words: &Words,
    ) -> ResponseResult<()> {
        let storage_ = storage.storage(target.chat.id);
        let category = category.clone();
        let position = *position;

        let Some(current_pattern) = read_category_filter_by_index(
            target,
            &storage_.categories(),
            &category,
            position,
            Some(
                CommandEditWordsFilter {
                    category: Some(category.clone()),
                    position: None,
                    page: None,
                    words: None,
                }
                .into(),
            ),
        )
        .await?
        else {
            return Ok(());
        };
        let words = extract_and_merge_words(
            &storage,
            target.chat.id,
            Words::read_pattern(&current_pattern),
        )
        .await;

        let prompt = |current_page: usize, total_pages: usize, total_words: usize| {
            markdown_format!(
                "✏️ Edit word filter **\\#{}** in category `{}`\n\n{}\n\nPage {}/{} \\({} words total\\)",
                position,
                category.as_str(),
                @raw if selected_words.as_ref().is_empty() { markdown_format!("_no words selected_") } else { markdown_format!("`{}`", selected_words.to_string()) },
                current_page,
                total_pages,
                total_words
            )
        };

        let word_command = |word: &str| {
            let mut new_words = selected_words.as_ref().clone();
            if new_words.contains(&word.to_string()) {
                new_words.retain(|w| w != word);
            } else {
                new_words.push(word.to_string());
            }
            CommandEditWordsFilter {
                category: Some(category.clone()),
                position: Some(position),
                page: Some(*page),
                words: Some(new_words.into()),
            }
            .into()
        };

        let page_command = |page_num: usize| {
            CommandEditWordsFilter {
                category: Some(category.clone()),
                position: Some(position),
                page: Some(page_num),
                words: Some(selected_words.clone()),
            }
            .into()
        };

        let apply_command = CommandEditFilter {
            category: Some(category.clone()),
            position: Some(position),
            pattern: selected_words.build_pattern(),
        }
        .into();

        select_word(
            target,
            prompt,
            words.as_ref(),
            selected_words.as_ref(),
            *page,
            word_command,
            page_command,
            apply_command,
            Some(
                CommandEditWordsFilter {
                    category: Some(category.clone()),
                    position: None,
                    page: None,
                    words: None,
                }
                .into(),
            ),
        )
        .await
    }
}

impl_command_io!(
    CommandEditWordsFilter,
    "edit_words_filter",
    ["<category>", "<position>", "<page>", "<words>"],
    category: Category,
    position: usize,
    page: usize,
    words: Words
);
impl_command_execute_4!(
    CommandEditWordsFilter,
    Arc<Stores>,
    category,
    position,
    page,
    words
);

impl From<CommandEditWordsFilter> for crate::commands::Command {
    fn from(cmd: CommandEditWordsFilter) -> Self {
        crate::commands::Command::EditWordsFilter(cmd)
    }
}
