use std::sync::Arc;

use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg},
    markdown_format, markdown_string,
};

use crate::{
    commands::Command,
    storages::{Category, CategoryStorageTrait},
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandAddCategory {
    pub category: Option<Category>,
}

impl CommandAddCategory {
    pub fn new(name: impl Into<String>) -> Self {
        CommandAddCategory {
            category: Category::from_string(&name.into()).ok(),
        }
    }
}

impl CommandTrait for CommandAddCategory {
    type A = Category;
    type B = EmptyArg;
    type C = EmptyArg;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<dyn CategoryStorageTrait>;

    const NAME: &'static str = "add_category";
    const PLACEHOLDERS: &[&'static str] = &["<name>"];

    fn from_arguments(
        a: Option<Self::A>,
        _: Option<Self::B>,
        _: Option<Self::C>,
        _: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandAddCategory { category: a }
    }

    fn param1(&self) -> Option<&Self::A> {
        self.category.as_ref()
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        _storage: Self::Context,
    ) -> teloxide::prelude::ResponseResult<()> {
        target
            .send_markdown_message(markdown_string!("➕ Add Category"))
            .await?;
        add_category_menu(target).await?;
        Ok(())
    }

    async fn run1(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        category: &Category,
    ) -> teloxide::prelude::ResponseResult<()> {
        match storage.add_category(target.chat.id, category).await {
            Ok(()) => {
                target
                    .send_markdown_message(markdown_format!(
                        "✅ Category `{}` created\\. Use {} to add regex patterns\\.",
                        category.as_str(),
                        Command::ADD_FILTER
                    ))
                    .await?;
            }
            Err(err_msg) => {
                target.send_markdown_message(err_msg).await?;
            }
        }
        Ok(())
    }
}

impl From<CommandAddCategory> for crate::commands::Command {
    fn from(cmd: CommandAddCategory) -> Self {
        crate::commands::Command::AddCategory(cmd)
    }
}

/// Show add category menu
pub async fn add_category_menu(target: &CommandReplyTarget) -> ResponseResult<()> {
    let text = markdown_string!(
        "➕ **Add a new category:**\n\nClick the button below and type the category name\\."
    );
    let keyboard = InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::switch_inline_query_current_chat(
            "➕ Add Category",
            CommandAddCategory::default().to_command_string(false),
        ),
    ]]);

    let message = target.markdown_message(text).await?;
    target
        .bot
        .edit_message_reply_markup(target.chat.id, message.id)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}
