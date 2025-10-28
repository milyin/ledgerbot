use std::{fmt::Display, str::FromStr, sync::Arc};

use teloxide::{prelude::ResponseResult, utils::command::ParseError};
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg, NoopCommand},
    markdown_format, markdown_string,
};

use crate::{
    commands::command_add_filter::CommandAddFilter,
    menus::{select_category::select_category, select_word::select_word},
    storage_traits::StorageTrait,
    utils::extract_words::extract_words,
};

/// Represents a collection of words separated by '|'
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Words(Vec<String>);

impl Words {
    pub fn new(words: Vec<String>) -> Self {
        Self(words)
    }

    pub fn as_vec(&self) -> &Vec<String> {
        &self.0
    }
}

impl Display for Words {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.join("|"))
    }
}

impl FromStr for Words {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let words = s.split('|').map(|w| w.trim().to_string()).collect();
        Ok(Words(words))
    }
}

impl AsRef<Vec<String>> for Words {
    fn as_ref(&self) -> &Vec<String> {
        &self.0
    }
}

impl AsMut<Vec<String>> for Words {
    fn as_mut(&mut self) -> &mut Vec<String> {
        &mut self.0
    }
}

impl From<Vec<String>> for Words {
    fn from(words: Vec<String>) -> Self {
        Words::new(words)
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandAddWordsFilter {
    pub category: Option<String>,
    pub page: Option<usize>,
    pub words: Option<Words>,
}

impl Words {
    pub fn build_pattern(&self) -> Option<String> {
        if self.as_ref().is_empty() {
            return None;
        }
        let escaped_words: Vec<String> = self.as_ref().iter().map(|w| regex::escape(w)).collect();
        Some(format!(r"(?i)\b({})\b", escaped_words.join("|")))
    }
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
                "💡 Select word\\(s\\) for filter in category `{}`\n\n`{}`\n\nPage {}/{} \\({} words total\\)",
                &category,
                if selected_words.as_ref().is_empty() { "∅".to_string() } else { selected_words.to_string() },
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
