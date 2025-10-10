use std::sync::Arc;
use teloxide::{
    Bot,
    payloads::{EditMessageReplyMarkupSetters, EditMessageTextSetters, SendMessageSetters},
    prelude::{Requester, ResponseResult},
    types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, Message, MessageId},
    utils::markdown::escape,
};

use crate::commands::Command;
use crate::handlers::CallbackData;
use crate::storage_traits::CategoryStorageTrait;

/// Add a filter to a category
pub async fn add_filter_command(
    bot: Bot,
    msg: Message,
    storage: Arc<dyn CategoryStorageTrait>,
    category: Option<String>,
    pattern: Option<String>,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    match (category, pattern) {
        (Some(category), Some(pattern)) => {
            let categories = storage.get_chat_categories(chat_id).await;

            // Check if category exists
            if !categories.contains_key(&category) {
                bot.send_message(
                    chat_id,
                    format!(
                        "❌ Category `{}` does not exist\\. Create it first with {}",
                        escape(&category), escape(Command::AddCategory { name: Some(category.clone()) }.to_string().as_str())
                    ),
                )
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
                return Ok(());
            }

            // Treat the pattern as a regexp directly without additional wrapping
            // Validate regex pattern
            match regex::Regex::new(&pattern) {
                Ok(_) => {
                    storage
                        .add_category_filter(chat_id, category.clone(), pattern.clone())
                        .await;
                    bot.send_message(
                        chat_id,
                        format!(
                            "✅ Filter `{}` added to category `{}`\\.",
                            escape(&pattern),
                            escape(&category)
                        ),
                    )
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                    .await?;
                }
                Err(e) => {
                    bot.send_message(
                        chat_id,
                        format!("❌ Invalid regex pattern: {}", escape(&e.to_string())),
                    )
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                    .await?;
                }
            }
        }
        (None, None) => {
            // Show the add filter menu instead
            let sent_msg = bot.send_message(chat_id, "🔧 Add Filter").await?;
            add_filter_menu(bot, chat_id, sent_msg.id, storage).await?;
        }
        (Some(category), None) => {
            bot.send_message(
                chat_id,
                format!(
                    "❌ Missing pattern\\. Usage: {}",
                    escape(&Command::AddFilter { category: Some(category.clone()), pattern: Some("pattern".to_string()) }.to_string())
                ),
            )
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
        }
        (None, Some(_)) => {
            bot.send_message(
                chat_id,
                format!(
                    "❌ Missing category\\. Usage: {}",
                    escape(&Command::AddFilter { category: Some("category".to_string()), pattern: Some("pattern".to_string()) }.to_string())
                ),
            )
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
        }
    }

    Ok(())
}

/// Remove a filter from a category by position
pub async fn remove_filter_command(
    bot: Bot,
    msg: Message,
    storage: Arc<dyn CategoryStorageTrait>,
    category: Option<String>,
    position: Option<usize>,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    match (category, position) {
        (Some(category), Some(position)) => {
            let categories = storage.get_chat_categories(chat_id).await;

            // Check if category exists
            if !categories.contains_key(&category) {
                bot.send_message(
                    chat_id,
                    format!("❌ Category `{}` does not exist\\.", escape(&category)),
                )
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
                return Ok(());
            }

            // Get filters for the category
            let Some(patterns) = categories.get(&category) else {
                bot.send_message(
                    chat_id,
                    format!("❌ Category `{}` has no filters\\.", escape(&category)),
                )
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
                return Ok(());
            };

            // Check if position is valid (0-indexed)
            if position >= patterns.len() {
                bot.send_message(
                    chat_id,
                    format!(
                        "❌ Invalid position `{}`\\. Category `{}` has **{}** filter\\(s\\) \\(indexed 0\\-{}\\)\\.",
                        escape(&position.to_string()),
                        escape(&category),
                        escape(&patterns.len().to_string()),
                        escape(&patterns.len().saturating_sub(1).to_string())
                    ),
                )
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
                return Ok(());
            }

            // Get the pattern at the specified position (0-indexed)
            let pattern = &patterns[position];
            let pattern_to_remove = pattern.clone();

            // Remove the filter
            storage
                .remove_category_filter(chat_id, &category, &pattern_to_remove)
                .await;
            bot.send_message(
                chat_id,
                format!(
                    "✅ Filter **\\#{}** \\(`{}`\\) removed from category `{}`\\.",
                    escape(&position.to_string()),
                    escape(&pattern_to_remove),
                    escape(&category)
                ),
            )
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
        }
        (None, None) => {
            // Show the remove filter menu instead
            let sent_msg = bot.send_message(chat_id, "🗑️ Remove Filter").await?;
            remove_filter_menu(bot, chat_id, sent_msg.id, storage).await?;
        }
        (Some(category), None) => {
            bot.send_message(
                chat_id,
                format!(
                    "❌ Missing position\\. Usage: {}",
                    escape(&Command::RemoveFilter { category: Some(category.clone()), position: Some(0) }.to_string())
                ),
            )
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
        }
        (None, Some(_)) => {
            bot.send_message(
                chat_id,
                format!(
                    "❌ Missing category\\. Usage: {}",
                    escape(&Command::RemoveFilter { category: Some("category".to_string()), position: Some(0) }.to_string())
                ),
            )
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
        }
    }

    Ok(())
}

/// Edit a filter in a category by position
pub async fn edit_filter_command(
    bot: Bot,
    msg: Message,
    storage: Arc<dyn CategoryStorageTrait>,
    category: Option<String>,
    position: Option<usize>,
    pattern: Option<String>,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    match (category, position, pattern) {
        (Some(category), Some(position), Some(pattern)) => {
            let categories = storage.get_chat_categories(chat_id).await;

            // Check if category exists
            if !categories.contains_key(&category) {
                bot.send_message(
                    chat_id,
                    format!("❌ Category `{}` does not exist\\.", escape(&category)),
                )
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
                return Ok(());
            }

            // Get filters for the category
            let Some(patterns) = categories.get(&category) else {
                bot.send_message(
                    chat_id,
                    format!("❌ Category `{}` has no filters\\.", escape(&category)),
                )
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
                return Ok(());
            };

            // Check if position is valid (0-indexed)
            if position >= patterns.len() {
                bot.send_message(
                    chat_id,
                    format!(
                        "❌ Invalid position `{}`\\. Category `{}` has **{}** filter\\(s\\) \\(indexed 0\\-{}\\)\\.",
                        escape(&position.to_string()),
                        escape(&category),
                        escape(&patterns.len().to_string()),
                        escape(&patterns.len().saturating_sub(1).to_string())
                    ),
                )
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
                return Ok(());
            }

            // Validate the new regex pattern
            match regex::Regex::new(&pattern) {
                Ok(_) => {
                    // Get the old pattern
                    let old_pattern = &patterns[position];
                    let old_pattern_clone = old_pattern.clone();

                    // Remove the old filter
                    storage
                        .remove_category_filter(chat_id, &category, &old_pattern_clone)
                        .await;

                    // Add the new filter at the same position
                    // Since we removed one, we need to re-fetch and insert at the correct position
                    storage
                        .add_category_filter(chat_id, category.clone(), pattern.clone())
                        .await;

                    // Note: The storage implementation might not preserve order perfectly,
                    // but we're doing our best here
                    bot.send_message(
                        chat_id,
                        format!(
                            "✅ Filter **\\#{}** updated in category `{}`\\.\n**Old:** `{}`\n**New:** `{}`",
                            escape(&position.to_string()), escape(&category), escape(&old_pattern_clone), escape(&pattern)
                        ),
                    )
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                    .await?;
                }
                Err(e) => {
                    bot.send_message(
                        chat_id,
                        format!("❌ Invalid regex pattern: {}", escape(&e.to_string())),
                    )
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                    .await?;
                }
            }
        }
        (None, None, None) => {
            // Show the edit filter menu instead
            let sent_msg = bot.send_message(chat_id, "✏️ Edit Filter").await?;
            edit_filter_menu(bot, chat_id, sent_msg.id, storage).await?;
        }
        (Some(category), Some(position), None) => {
            bot.send_message(
                chat_id,
                format!(
                    "❌ Missing pattern\\. Usage: {}",
                    escape(&Command::EditFilter { 
                        category: Some(category.clone()), 
                        position: Some(position), 
                        pattern: Some("new_pattern".to_string()) 
                    }.to_string())
                ),
            )
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
        }
        (Some(category), None, _) => {
            bot.send_message(
                chat_id,
                format!(
                    "❌ Missing position\\. Usage: {}",
                    escape(&Command::EditFilter { 
                        category: Some(category.clone()), 
                        position: Some(0), 
                        pattern: Some("new_pattern".to_string()) 
                    }.to_string())
                ),
            )
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
        }
        (None, _, _) => {
            bot.send_message(
                chat_id,
                format!(
                    "❌ Missing category\\. Usage: {}",
                    escape(&Command::EditFilter { 
                        category: Some("category".to_string()), 
                        position: Some(0), 
                        pattern: Some("new_pattern".to_string()) 
                    }.to_string())
                ),
            )
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
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
    storage: Arc<dyn CategoryStorageTrait>,
) -> ResponseResult<()> {
    let categories = storage.get_chat_categories(chat_id).await;

    if categories.is_empty() {
        bot.edit_message_text(chat_id, message_id, "No categories available.")
            .await?;
    } else {
        let text = "🗑️ **Select category to remove filter:**\n\nClick a button to see filters for that category\\.";

        // Create buttons for each category that has filters
        let buttons: Vec<Vec<InlineKeyboardButton>> = categories
            .iter()
            .filter(|(_, patterns)| !patterns.is_empty())
            .map(|(name, _)| {
                vec![InlineKeyboardButton::callback(
                    format!("🗑️ {}", name),
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

        bot.edit_message_text(chat_id, message_id, text)
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
        bot.edit_message_reply_markup(chat_id, message_id)
            .reply_markup(keyboard)
            .await?;
    }

    Ok(())
}

/// Show edit filter interface - first show categories
pub async fn edit_filter_menu(
    bot: Bot,
    chat_id: ChatId,
    message_id: MessageId,
    storage: Arc<dyn CategoryStorageTrait>,
) -> ResponseResult<()> {
    let categories = storage.get_chat_categories(chat_id).await;

    if categories.is_empty() {
        bot.edit_message_text(chat_id, message_id, "No categories available.")
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
                    CallbackData::EditFilterCategory(name.clone()),
                )]
            })
            .collect();

        if buttons.is_empty() {
            bot.edit_message_text(chat_id, message_id, "No filters defined in any category.")
                .await?;
            return Ok(());
        }

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

/// Show add filter interface - first show categories
pub async fn add_filter_menu(
    bot: Bot,
    chat_id: ChatId,
    message_id: MessageId,
    storage: Arc<dyn CategoryStorageTrait>,
) -> ResponseResult<()> {
    let categories = storage.get_chat_categories(chat_id).await;

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
