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

        let category = category.clone();

        // Show word selection menu with pagination
        select_word(
            target,
            |current_page, total_pages, total_words| {
                markdown_format!(
                    "💡 Select word\\(s\\) for filter in category `{}`\n\nPage {}/{} \\({} words total\\)",
                    &category,
                    current_page,
                    total_pages,
                    total_words
                )
            },
            &words,
            *page,
            |_word| NoopCommand,
            |page_num| CommandAddFilter2 {
                category: Some(category.clone()),
                page: Some(page_num),
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
