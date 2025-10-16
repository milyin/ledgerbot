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
    type J = EmptyArg;

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
        _: Option<Self::J>,
    ) -> Self {
        CommandAddCategory { name: a }
    }

    fn param0(&self) -> Option<&Self::A> {
        self.name.as_ref()
    }
    fn param1(&self) -> Option<&Self::B> {
        None
    }

    fn param2(&self) -> Option<&Self::C> {
        None
    }

    fn param3(&self) -> Option<&Self::D> {
        None
    }

    fn param4(&self) -> Option<&Self::E> {
        None
    }

    fn param5(&self) -> Option<&Self::F> {
        None
    }

    fn param6(&self) -> Option<&Self::G> {
        None
    }

    fn param7(&self) -> Option<&Self::H> {
        None
    }

    fn param8(&self) -> Option<&Self::I> {
        None
    }

    fn param9(&self) -> Option<&Self::J> {
        None
    }

    async fn run(
        &self,
        bot: Bot,
        msg: Message,
        storage: Self::Context,
    ) -> teloxide::prelude::ResponseResult<()> {
        let chat_id = msg.chat.id;

        // Check if name is provided
        match &self.name {
            None => {
                // Show the add category menu instead
                let sent_msg = bot
                    .send_markdown_message(chat_id, markdown_string!("➕ Add Category"))
                    .await?;
                add_category_menu(bot, chat_id, sent_msg.id).await?;
            }
            Some(name) => match storage.add_category(chat_id, name.clone()).await {
                Ok(()) => {
                    bot.send_markdown_message(
                        chat_id,
                        markdown_format!(
                            "✅ Category `{}` created\\. Use {} to add regex patterns\\.",
                            name,
                            Command::ADD_FILTER
                        ),
                    )
                    .await?;
                }
                Err(err_msg) => {
                    bot.send_markdown_message(chat_id, markdown_format!("ℹ️ {}", &err_msg))
                        .await?;
                }
            },
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
            format!("{} ", CommandAddCategory::NAME),
        ),
    ]]);

    bot.edit_markdown_message_text(chat_id, message_id, text)
        .await?;
    bot.edit_message_reply_markup(chat_id, message_id)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}
