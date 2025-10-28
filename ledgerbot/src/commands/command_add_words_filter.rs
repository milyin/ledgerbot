use std::sync::Arc;

use teloxide::prelude::ResponseResult;
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg, NoopCommand},
    markdown_format, markdown_string,
};

use crate::{
    commands::command_add_filter::CommandAddFilter,
    menus::{
        select_category::select_category,
        select_word::{select_word, Words},
    },
    storage_traits::StorageTrait,
    utils::extract_words::extract_words,
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandAddWordsFilter {
    pub category: Option<String>,
    pub page: Option<usize>,
    pub words: Option<Words>,
}

impl CommandTrait for CommandAddWordsFilter {
    type A = String;
    type B = usize;
    type C = Words;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<dyn StorageTrait>;

    const NAME: &'static str = "add_words_filter";
    const PLACEHOLDERS: &[&'static str] = &["<category>", "<page>", "<words>"];

    fn from_arguments(
        category: Option<Self::A>,
        page: Option<Self::B>,
        words: Option<Self::C>,
        _: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandAddWordsFilter {
            category,
            page,
            words,
        }
    }

    fn param1(&self) -> Option<&Self::A> {
        self.category.as_ref()
    }

    fn param2(&self) -> Option<&Self::B> {
        self.page.as_ref()
    }

    fn param3(&self) -> Option<&Self::C> {
        self.words.as_ref()
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
    ) -> ResponseResult<()> {
        select_category(
            target,
            &storage.as_category_storage(),
            markdown_string!("➕ Select Category to add filter"),
            |name| CommandAddWordsFilter {
                category: Some(name.to_string()),
                page: Some(0),
                words: None,
            },
            None::<NoopCommand>,
        )
        .await
    }

    async fn run1(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        category: &String,
    ) -> ResponseResult<()> {
        // Default to page 0 when no page specified
        self.run3(target, storage, category, &0, &Words::default())
            .await
    }

    async fn run2(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        category: &String,
        page: &usize,
    ) -> ResponseResult<()> {
        self.run3(target, storage, category, page, &Words::default())
            .await
    }

    async fn run3(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        category: &String,
        page: &usize,
        selected_words: &Words,
    ) -> ResponseResult<()> {
        // Get expenses and categories
        let expenses = storage
            .clone()
            .as_expense_storage()
            .get_chat_expenses(target.chat.id)
            .await;
        let categories = storage
            .clone()
            .as_category_storage()
            .get_chat_categories(target.chat.id)
            .await
            .unwrap_or_default();

        // Extract words from uncategorized expenses
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

        // Show word selection menu with pagination
        let prompt = |current_page: usize, total_pages: usize, total_words: usize| {
            markdown_format!(
                "💡 Select word\\(s\\) for filter in category `{}`\n\n{}\n\nPage {}/{} \\({} words total\\)",
                &category,
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
        };

        let page_command = |page_num: usize| CommandAddWordsFilter {
            category: Some(category.clone()),
            page: Some(page_num),
            words: Some(selected_words.clone()),
        };

        // Build regex pattern from selected words
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
            },
            Some(CommandAddWordsFilter {
                category: None,
                page: None,
                words: None,
            }),
        )
        .await
    }
}

impl From<CommandAddWordsFilter> for crate::commands::Command {
    fn from(cmd: CommandAddWordsFilter) -> Self {
        crate::commands::Command::AddWordsFilter(cmd)
    }
}
