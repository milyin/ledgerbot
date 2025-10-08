use teloxide::{
    Bot,
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
    types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, Message, MessageId},
};

use crate::handlers::CallbackData;
use crate::storage::{CategoryStorage, add_category_filter, get_chat_categories};

/// Add a filter to a category
pub async fn add_filter_command(
    bot: Bot,
    msg: Message,
    category_storage: CategoryStorage,
    category: Option<String>,
    pattern: Option<String>,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    match (category, pattern) {
        (Some(category), Some(pattern)) => {
            let categories = get_chat_categories(&category_storage, chat_id).await;

            // Check if category exists
            if !categories.contains_key(&category) {
                bot.send_message(
                    chat_id,
                    format!(
                        "❌ Category '{}' does not exist. Create it first with /add_category {}",
                        category, category
                    ),
                )
                .await?;
                return Ok(());
            }

            // Treat the pattern as a regexp directly without additional wrapping
            // Validate regex pattern
            match regex::Regex::new(&pattern) {
                Ok(_) => {
                    add_category_filter(
                        &category_storage,
                        chat_id,
                        category.clone(),
                        pattern.clone(),
                    )
                    .await;
                    bot.send_message(
                        chat_id,
                        format!("✅ Filter '{}' added to category '{}'.", pattern, category),
                    )
                    .await?;
                }
                Err(e) => {
                    bot.send_message(chat_id, format!("❌ Invalid regex pattern: {}", e))
                        .await?;
                }
            }
        }
        (None, None) => {
            // Show the add filter menu instead
            let sent_msg = bot.send_message(chat_id, "🔧 Add Filter").await?;
            add_filter_menu(bot, chat_id, sent_msg.id, category_storage).await?;
        }
        (Some(category), None) => {
            bot.send_message(
                chat_id,
                format!(
                    "❌ Missing pattern. Usage: /add_filter {} <pattern>",
                    category
                ),
            )
            .await?;
        }
        (None, Some(_)) => {
            bot.send_message(
                chat_id,
                "❌ Missing category. Usage: /add_filter <category> <pattern>",
            )
            .await?;
        }
    }

    Ok(())
}

/// Remove a filter from a category
pub async fn remove_filter_command(
    bot: Bot,
    msg: Message,
    category_storage: CategoryStorage,
    category: Option<String>,
    pattern: Option<String>,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    match (category, pattern) {
        (Some(category), Some(pattern)) => {
            let categories = get_chat_categories(&category_storage, chat_id).await;

            // Check if category exists
            if !categories.contains_key(&category) {
                bot.send_message(
                    chat_id,
                    format!("❌ Category '{}' does not exist.", category),
                )
                .await?;
                return Ok(());
            }

            // Check if filter exists in the category
            if let Some(patterns) = categories.get(&category)
                && !patterns.contains(&pattern)
            {
                bot.send_message(
                    chat_id,
                    format!(
                        "❌ Filter '{}' not found in category '{}'.",
                        pattern, category
                    ),
                )
                .await?;
                return Ok(());
            }

            // Remove the filter
            crate::storage::remove_category_filter(&category_storage, chat_id, &category, &pattern)
                .await;
            bot.send_message(
                chat_id,
                format!(
                    "✅ Filter '{}' removed from category '{}'.",
                    pattern, category
                ),
            )
            .await?;
        }
        (None, None) => {
            // Show the remove filter menu instead
            let sent_msg = bot.send_message(chat_id, "🗑️ Remove Filter").await?;
            remove_filter_menu(bot, chat_id, sent_msg.id, category_storage).await?;
        }
        (Some(category), None) => {
            bot.send_message(
                chat_id,
                format!(
                    "❌ Missing pattern. Usage: /remove_filter {} <pattern>",
                    category
                ),
            )
            .await?;
        }
        (None, Some(_)) => {
            bot.send_message(
                chat_id,
                "❌ Missing category. Usage: /remove_filter <category> <pattern>",
            )
            .await?;
        }
    }

    Ok(())
}

/// Show remove filter interface - first show categories
pub async fn remove_filter_menu(
    bot: Bot,
    chat_id: ChatId,
    message_id: MessageId,
    category_storage: CategoryStorage,
) -> ResponseResult<()> {
    let categories = get_chat_categories(&category_storage, chat_id).await;

    if categories.is_empty() {
        bot.edit_message_text(chat_id, message_id, "No categories available.")
            .await?;
    } else {
        let text = "�️ **Select category to remove filter:**\n\nClick a button to see filters for that category.";

        // Create buttons for each category that has filters
        let buttons: Vec<Vec<InlineKeyboardButton>> = categories
            .iter()
            .filter(|(_, patterns)| !patterns.is_empty())
            .map(|(name, _)| {
                vec![InlineKeyboardButton::callback(
                    format!("�️ {}", name),
                    CallbackData::RemoveFilterCategory(name.clone()),
                )]
            })
            .collect();

        if buttons.is_empty() {
            bot.edit_message_text(chat_id, message_id, "No filters defined in any category.")
                .await?;
            return Ok(());
        }

        let keyboard = InlineKeyboardMarkup::new(buttons);

        bot.edit_message_text(chat_id, message_id, text).await?;
        bot.edit_message_reply_markup(chat_id, message_id)
            .reply_markup(keyboard)
            .await?;
    }

    Ok(())
}

/// Show add filter interface - first show categories
pub async fn add_filter_menu(
    bot: Bot,
    chat_id: ChatId,
    message_id: MessageId,
    category_storage: CategoryStorage,
) -> ResponseResult<()> {
    let categories = get_chat_categories(&category_storage, chat_id).await;

    if categories.is_empty() {
        bot.edit_message_text(
            chat_id,
            message_id,
            "No categories available. Create a category first with /add_category <name>",
        )
        .await?;
    } else {
        let text = "� **Select category to add filter:**";

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

        bot.edit_message_text(chat_id, message_id, text).await?;
        bot.edit_message_reply_markup(chat_id, message_id)
            .reply_markup(keyboard)
            .await?;
    }

    Ok(())
}
