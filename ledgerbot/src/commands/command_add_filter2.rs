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
}

impl CommandTrait for CommandAddFilter2 {
    type A = String;
    type B = EmptyArg;
    type C = EmptyArg;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<dyn StorageTrait>;

    const NAME: &'static str = "add_filter2";
    const PLACEHOLDERS: &[&'static str] = &["<category>"];

    fn from_arguments(
        category: Option<Self::A>,
        _: Option<Self::B>,
        _: Option<Self::C>,
        _: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandAddFilter2 { category }
    }

    fn param1(&self) -> Option<&Self::A> {
        self.category.as_ref()
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

        // Show word selection menu
        select_word(
            target,
            markdown_format!(
                "💡 Select word\\(s\\) for filter in category `{}`",
                category
            ),
            &words,
            |_word| NoopCommand,
            Some(CommandAddFilter2 {
                category: None,
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
