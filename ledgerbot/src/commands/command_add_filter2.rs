use std::sync::Arc;

use teloxide::prelude::ResponseResult;
use yoroolbot::{markdown_format, markdown_string};

use crate::{
    commands::command_trait::{CommandReplyTarget, CommandTrait, EmptyArg, NoopCommand},
    menus::{select_category::select_category, select_word::select_word},
    storage_traits::StorageTrait,
    utils::extract_words::extract_words,
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandAddFilter2 {
    pub category: Option<String>,
    pub page: Option<usize>,
}

impl CommandTrait for CommandAddFilter2 {
    type A = String;
    type B = usize;
    type C = EmptyArg;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<dyn StorageTrait>;

    const NAME: &'static str = "add_filter2";
    const PLACEHOLDERS: &[&'static str] = &["<category>", "<page>"];

    fn from_arguments(
        category: Option<Self::A>,
        page: Option<Self::B>,
        _: Option<Self::C>,
        _: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandAddFilter2 { category, page }
    }

    fn param1(&self) -> Option<&Self::A> {
        self.category.as_ref()
    }

    fn param2(&self) -> Option<&Self::B> {
        self.page.as_ref()
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
            |name| CommandAddFilter2 {
                category: Some(name.to_string()),
                page: Some(0),
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
        self.run2(target, storage, category, &0).await
    }

    async fn run2(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        category: &String,
        page: &usize,
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
            .await;

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

        // Pagination constants
        const WORDS_PER_PAGE: usize = 20;
        let total_words = words.len();
        let total_pages = total_words.div_ceil(WORDS_PER_PAGE);
        let current_page = *page;

        // Ensure page is within bounds
        let page_number = current_page.min(total_pages.saturating_sub(1));
        let page_offset = page_number * WORDS_PER_PAGE;

        // Get words for current page
        let page_words: Vec<String> = words
            .iter()
            .skip(page_offset)
            .take(WORDS_PER_PAGE)
            .cloned()
            .collect();

        // Show word selection menu with pagination
        select_word(
            target,
            markdown_format!(
                "💡 Select word\\(s\\) for filter in category `{}`\n\nPage {}/{} \\({} words total\\)",
                category,
                page_number + 1,
                total_pages,
                total_words
            ),
            &page_words,
            |_word| NoopCommand,
            // Previous page button (or inactive if first page)
            if page_number > 0 {
                Some(CommandAddFilter2 {
                    category: Some(category.clone()),
                    page: Some(page_number - 1),
                })
            } else {
                None
            },
            // Next page button (or inactive if last page)
            if page_offset + WORDS_PER_PAGE < total_words {
                Some(CommandAddFilter2 {
                    category: Some(category.clone()),
                    page: Some(page_number + 1),
                })
            } else {
                None
            },
            Some(CommandAddFilter2 {
                category: None,
                page: None,
            }),
        )
        .await
    }
}

impl From<CommandAddFilter2> for crate::commands::Command {
    fn from(cmd: CommandAddFilter2) -> Self {
        crate::commands::Command::AddFilter2(cmd)
    }
}
