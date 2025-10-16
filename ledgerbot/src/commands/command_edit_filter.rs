use std::sync::Arc;

use teloxide::{
    payloads::EditMessageReplyMarkupSetters, prelude::{Requester, ResponseResult}, types::{Chat, ChatId, InlineKeyboardButton, InlineKeyboardMarkup, MessageId}, Bot
};
use yoroolbot::{markdown::MarkdownStringMessage, markdown_format};

use crate::{
    commands::command_trait::{CommandTrait, EmptyArg}, storage_traits::CategoryStorageTrait
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandEditFilter {
    pub category: Option<String>,
    pub position: Option<usize>,
    pub pattern: Option<String>,
}

impl CommandTrait for CommandEditFilter {
    type A = String;
    type B = usize;
    type C = String;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<dyn CategoryStorageTrait>;

    const NAME: &'static str = "edit_filter";
    const PLACEHOLDERS: &[&'static str] = &["<category>", "<position>", "<new_pattern>"];

    fn from_arguments(
        a: Option<Self::A>,
        b: Option<Self::B>,
        c: Option<Self::C>,
        _: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandEditFilter {
            category: a,
            position: b,
            pattern: c,
        }
    }

    fn param1(&self) -> Option<&Self::A> {
        self.category.as_ref()
    }
    fn param2(&self) -> Option<&Self::B> {
        self.position.as_ref()
    }
    fn param3(&self) -> Option<&Self::C> {
        self.pattern.as_ref()
    }

    async fn run0(&self, bot: Bot, chat: Chat, _msg_id: Option<MessageId>, storage: Self::Context) -> ResponseResult<()> {
        let sent_msg = bot.markdown_message(chat.id, None, markdown_format!("✏️ Edit Filter")).await?;
        select_category_menu(bot, chat.id, sent_msg.id, storage).await?;
        Ok(())
    }
}

pub async fn select_category_menu(
    bot: Bot,
    chat_id: ChatId,
    message_id: MessageId,
    storage: Arc<dyn CategoryStorageTrait>,
) -> ResponseResult<()> {
    let categories = storage.get_chat_categories(chat_id).await;

    if categories.is_empty() {
        bot.markdown_message(chat_id, Some(message_id), markdown_format!("No categories available"))
            .await?;
    } else {
        let text = "✏️ **Select category to edit filter:**\n\nClick a button to see filters for that category\\.";

        // Create buttons for each category that has filters
        let buttons: Vec<Vec<InlineKeyboardButton>> = categories
            .iter()
            .filter(|(_, patterns)| !patterns.is_empty())
            .map(|(name, _)| {
                vec![InlineKeyboardButton::callback(
                    format!("✏️ {}", name),
                    CommandEditFilter {
                        category: Some(name.clone()),
                        position: None,
                        pattern: None,
                    }.to_command_string(false)
                )]
            })
            .collect();

        if buttons.is_empty() {
            bot.markdown_message(chat_id, Some(message_id), markdown_format!("No filters defined in any category"))
                .await?;
            return Ok(());
        }

        let keyboard = InlineKeyboardMarkup::new(buttons);

        bot.markdown_message(chat_id, Some(message_id), markdown_format!("{}", text))
            .await?;
        bot.edit_message_reply_markup(chat_id, message_id)
            .reply_markup(keyboard)
            .await?;
    }

    Ok(())
}

