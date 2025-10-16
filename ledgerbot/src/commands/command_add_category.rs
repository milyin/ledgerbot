use std::sync::Arc;

use teloxide::{
    Bot,
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Message, Requester, ResponseResult},
    types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, MessageId},
};
use yoroolbot::{markdown::MarkdownStringMessage, markdown_format, markdown_string};

use crate::{
    commands::{
        Command,
        command_trait::{CommandTrait, EmptyArg},
    },
    storage_traits::CategoryStorageTrait,
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandAddCategory {
    pub name: Option<String>,
}

impl CommandAddCategory {
    pub fn new(name: impl Into<String>) -> Self {
        CommandAddCategory {
            name: Some(name.into()),
        }
    }
}

impl CommandTrait for CommandAddCategory {
    type A = String;
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
        CommandAddCategory { name: a }
    }

    fn param1(&self) -> Option<&Self::A> {
        self.name.as_ref()
    }

    async fn run0(
        &self,
        bot: Bot,
        msg: Message,
        _storage: Self::Context,
    ) -> teloxide::prelude::ResponseResult<()> {
        let chat_id = msg.chat.id;
        let sent_msg = bot
            .send_markdown_message(chat_id, markdown_string!("➕ Add Category"))
            .await?;
        add_category_menu(bot, chat_id, sent_msg.id).await?;
        Ok(())
    }

    async fn run1(
        &self,
        bot: Bot,
        msg: Message,
        storage: Self::Context,
        name: &String,
    ) -> teloxide::prelude::ResponseResult<()> {
        match storage.add_category(msg.chat.id, name.clone()).await {
            Ok(()) => {
                bot.send_markdown_message(
                    msg.chat.id,
                    markdown_format!(
                        "✅ Category `{}` created\\. Use {} to add regex patterns\\.",
                        name,
                        Command::ADD_FILTER
                    ),
                )
                .await?;
            }
            Err(err_msg) => {
                bot.send_markdown_message(msg.chat.id, markdown_format!("ℹ️ {}", &err_msg))
                    .await?;
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
pub async fn add_category_menu(
    bot: Bot,
    chat_id: ChatId,
    message_id: MessageId,
) -> ResponseResult<()> {
    let text = markdown_string!(
        "➕ **Add a new category:**\n\nClick the button below and type the category name\\."
    );
    let keyboard = InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::switch_inline_query_current_chat(
            "➕ Add Category",
            CommandAddCategory::default().to_command_string(false),
        ),
    ]]);

    bot.edit_markdown_message_text(chat_id, message_id, text)
        .await?;
    bot.edit_message_reply_markup(chat_id, message_id)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}
