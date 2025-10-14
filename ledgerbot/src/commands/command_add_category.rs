use std::{fmt::Display, sync::Arc};

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
        command_trait::{
            CommandTrait, EmptyArg1, EmptyArg2, EmptyArg3, EmptyArg4, EmptyArg5, EmptyArg6,
            EmptyArg7, EmptyArg8, EmptyArg9,
        },
    },
    storage_traits::CategoryStorageTrait,
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandAddCategory {
    pub name: Option<String>,
}

impl CommandAddCategory {
    pub fn new(name: String) -> Self {
        CommandAddCategory { name: Some(name) }
    }
}

impl CommandTrait for CommandAddCategory {
    type A = String;
    type B = EmptyArg1<1>;
    type C = EmptyArg2<1>;
    type D = EmptyArg3<1>;
    type E = EmptyArg4<1>;
    type F = EmptyArg5<1>;
    type G = EmptyArg6<1>;
    type H = EmptyArg7<1>;
    type I = EmptyArg8<1>;
    type J = EmptyArg9<1>;

    type Context = Arc<dyn CategoryStorageTrait>;

    const NAME: &'static str = "add_category";

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

impl From<CommandAddCategory> for String {
    fn from(cmd: CommandAddCategory) -> Self {
        format!(
            "{} {}",
            CommandAddCategory::NAME,
            cmd.name.unwrap_or("<name>".into())
        )
    }
}

impl Display for CommandAddCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self.clone()))
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
