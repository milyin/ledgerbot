use std::sync::Arc;

use teloxide::{
    Bot,
    payloads::{EditMessageReplyMarkupSetters, EditMessageTextSetters},
    prelude::{Requester, ResponseResult},
    types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, MessageId},
};

use crate::{handlers::CallbackData, storage_traits::CategoryStorageTrait};

/// Show add filter interface - first show categories
pub async fn add_filter_menu(
    bot: Bot,
    chat_id: ChatId,
    message_id: MessageId,
    storage: Arc<dyn CategoryStorageTrait>,
) -> ResponseResult<()> {
    let categories = storage
        .get_chat_categories(chat_id)
        .await
        .unwrap_or_default();

    if categories.is_empty() {
        bot.edit_message_text(
            chat_id,
            message_id,
            "No categories available\\. Create a category first with /add\\_category <name>",
        )
        .await?;
    } else {
        let text = "🔧 **Select category to add filter:**";

        // Create buttons for each category
        let buttons: Vec<Vec<InlineKeyboardButton>> = categories
            .keys()
            .map(|name| {
                vec![InlineKeyboardButton::callback(
                    format!("🔧 {}", name),
                    CallbackData::AddFilterCategory(name.clone()),
                )]
            })
            .collect();

        let keyboard = InlineKeyboardMarkup::new(buttons);

        bot.edit_message_text(chat_id, message_id, text)
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
        bot.edit_message_reply_markup(chat_id, message_id)
            .reply_markup(keyboard)
            .await?;
    }

    Ok(())
}
