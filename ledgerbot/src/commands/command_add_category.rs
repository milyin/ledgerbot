use std::sync::Arc;

use serde::{Deserialize, Serialize};
use telluride::{markdown_format, markdown_string};
use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};

use crate::{
    commands::Command,
    impl_command_execute_1, impl_command_io,
    storages::{Category, CategoryStorageTrait},
    ui::CommandContext,
};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandAddCategory {
    pub category: Option<Category>,
}

impl CommandAddCategory {
    pub fn new(name: impl Into<String>) -> Self {
        CommandAddCategory {
            category: Category::from_string(&name.into()).ok(),
        }
    }

    async fn run0(
        &self,
        target: &CommandContext,
        _storage: Arc<dyn CategoryStorageTrait>,
    ) -> ResponseResult<()> {
        target
            .send_markdown_message(markdown_string!("➕ Add Category"))
            .await?;
        add_category_menu(target).await?;
        Ok(())
    }

    async fn run1(
        &self,
        target: &CommandContext,
        storage: Arc<dyn CategoryStorageTrait>,
        category: &Category,
    ) -> ResponseResult<()> {
        match storage.add_category(category).await {
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

impl_command_io!(CommandAddCategory, "add_category", ["<name>"], category: Category);
impl_command_execute_1!(CommandAddCategory, Arc<dyn CategoryStorageTrait>, category);

impl From<CommandAddCategory> for crate::commands::Command {
    fn from(cmd: CommandAddCategory) -> Self {
        crate::commands::Command::AddCategory(cmd)
    }
}

pub async fn add_category_menu(target: &CommandContext) -> ResponseResult<()> {
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
