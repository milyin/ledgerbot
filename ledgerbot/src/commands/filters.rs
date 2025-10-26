use std::sync::Arc;

use teloxide::{
    Bot,
    payloads::{EditMessageReplyMarkupSetters, EditMessageTextSetters, SendMessageSetters},
    prelude::{Requester, ResponseResult},
    types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, Message, MessageId},
    utils::markdown::escape,
};
use yoroolbot::{markdown::MarkdownStringMessage, markdown_format};

use crate::{
    commands::{Command, command_add_category::CommandAddCategory, command_trait::CommandTrait},
    handlers::CallbackData,
    storage_traits::CategoryStorageTrait,
};

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
                bot.markdown_message(
                    chat_id,
                    None,
                    markdown_format!(
                        "❌ Category `{}` does not exist\\. Create it first with {}",
                        &category,
                        CommandAddCategory::new(category).to_command_string(true)
                    ),
                )
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
                    bot.markdown_message(
                        chat_id,
                        None,
                        markdown_format!("❌ Invalid regex pattern: `{}`", e.to_string()),
                    )
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
            bot.markdown_message(
                chat_id,
                None,
                markdown_format!(
                    "❌ Missing pattern\\. Usage: {}",
                    &Command::AddFilter {
                        category: Some(category.clone()),
                        pattern: Some("pattern".to_string())
                    }
                    .to_string()
                ),
            )
            .await?;
        }
        (None, Some(_)) => {
            bot.markdown_message(
                chat_id,
                None,
                markdown_format!(
                    "❌ Missing category\\. Usage: {}",
                    escape(
                        &Command::AddFilter {
                            category: Some("category".to_string()),
                            pattern: Some("pattern".to_string())
                        }
                        .to_string()
                    )
                ),
            )
            .await?;
        }
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
