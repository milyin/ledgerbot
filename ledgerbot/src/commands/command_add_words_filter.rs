use std::sync::Arc;

use serde::{Deserialize, Serialize};
use telluride::{markdown_format, markdown_string};
use teloxide::prelude::ResponseResult;

use crate::{
    commands::command_add_filter::CommandAddFilter,
    impl_command_execute_3, impl_command_io,
    menus::{
        select_category::select_category,
        select_word::{Words, select_word},
    },
    storages::{Category, Expense, Storage},
    ui::CommandContext,
    utils::extract_words::extract_words,
};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandAddWordsFilter {
    pub category: Option<Category>,
    pub page: Option<usize>,
    pub words: Option<Words>,
}

impl CommandAddWordsFilter {
    async fn run0(&self, target: &CommandContext, storage: Arc<Storage>) -> ResponseResult<()> {
        select_category(
            target,
            &storage.categories(),
            markdown_string!("➕ Select Category to add filter"),
            |category| {
                CommandAddWordsFilter {
                    category: Some(category.clone()),
                    page: Some(0),
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
        storage: Arc<Storage>,
        category: &Category,
    ) -> ResponseResult<()> {
        self.run3(target, storage, category, &0, &Words::default())
            .await
    }

    async fn run2(
        &self,
        target: &CommandContext,
        storage: Arc<Storage>,
        category: &Category,
        page: &usize,
    ) -> ResponseResult<()> {
        self.run3(target, storage, category, page, &Words::default())
            .await
    }

    async fn run3(
        &self,
        target: &CommandContext,
        storage: Arc<Storage>,
        category: &Category,
        page: &usize,
        selected_words: &Words,
    ) -> ResponseResult<()> {
        let all_expenses = storage.expenses().get_all_expenses().await;
        let expenses: Vec<Expense> = all_expenses
            .into_iter()
            .map(|(_, expense)| expense)
            .collect();

        let categories = storage
            .categories()
            .get_categories()
            .await
            .unwrap_or_default();

        let words = extract_words(&expenses, &categories);

        if words.is_empty() {
            target
                .send_markdown_message(markdown_format!(
                    "💡 No uncategorized expenses found\\. All expenses are already categorized\\."
                ))
                .await?;
            return Ok(());
        }

        let category = category.clone();

        let prompt = |current_page: usize, total_pages: usize, total_words: usize| {
            markdown_format!(
                "💡 Select word\\(s\\) for filter in category `{}`\n\n{}\n\nPage {}/{} \\({} words total\\)",
                category.as_str(),
                @raw if selected_words.as_ref().is_empty() { markdown_format!("_no words selected_") } else { markdown_format!("`{}`", selected_words.to_string()) },
                current_page,
                total_pages,
                total_words
            )
        };

        let word_command = |word: &str| {
            let mut selected_words = selected_words.as_ref().clone();
            if selected_words.contains(&word.to_string()) {
                selected_words.retain(|w| w != word);
            } else {
                selected_words.push(word.to_string());
            }
            CommandAddWordsFilter {
                category: Some(category.clone()),
                page: Some(*page),
                words: Some(selected_words.into()),
            }
            .into()
        };

        let page_command = |page_num: usize| {
            CommandAddWordsFilter {
                category: Some(category.clone()),
                page: Some(page_num),
                words: Some(selected_words.clone()),
            }
            .into()
        };

        select_word(
            target,
            prompt,
            &words,
            selected_words.as_ref(),
            *page,
            word_command,
            page_command,
            CommandAddFilter {
                category: Some(category.clone()),
                pattern: selected_words.build_pattern(),
            }
            .into(),
            Some(
                CommandAddWordsFilter {
                    category: None,
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
    CommandAddWordsFilter,
    "add_words_filter",
    ["<category>", "<page>", "<words>"],
    category: Category,
    page: usize,
    words: Words
);
impl_command_execute_3!(CommandAddWordsFilter, Arc<Storage>, category, page, words);

impl From<CommandAddWordsFilter> for crate::commands::Command {
    fn from(cmd: CommandAddWordsFilter) -> Self {
        crate::commands::Command::AddWordsFilter(cmd)
    }
}
