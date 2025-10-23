use std::sync::Arc;

use teloxide::{
    Bot,
    payloads::{EditMessageReplyMarkupSetters, EditMessageTextSetters},
    prelude::{Requester, ResponseResult},
    types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, Message, MessageId},
    utils::markdown::escape,
};
use yoroolbot::{markdown::MarkdownStringMessage, markdown_format};

use crate::{
    commands::{
        Command, command_add_category::CommandAddCategory, command_edit_filter::CommandEditFilter,
        command_trait::CommandTrait,
    },
    handlers::CallbackData,
    storage_traits::CategoryStorageTrait,
};

/// List all categories as executable commands
pub async fn categories_command(
    bot: Bot,
    msg: Message,
    storage: Arc<dyn CategoryStorageTrait>,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let categories = storage.get_chat_categories(chat_id).await;

    if categories.is_empty() {
        bot.markdown_message(
            chat_id,
            None,
            markdown_format!(
                "📂 No categories defined yet\\. Use {} to create one\\.",
                CommandAddCategory::default().to_command_string(true)
            ),
        )
        .await?;
    } else {
        let mut result = String::new();

        // Sort categories for consistent output
        let mut sorted_categories: Vec<_> = categories.iter().collect();
        sorted_categories.sort_by(|a, b| a.0.cmp(b.0));

        for (name, patterns) in sorted_categories {
            // First create the category
            result.push_str(&CommandAddCategory::new(name).to_command_string(true));
            result.push('\n');

            // Then assign patterns if they exist
            for pattern in patterns {
                result.push_str(
                    &Command::AddFilter {
                        category: Some(name.clone()),
                        pattern: Some(pattern.clone()),
                    }
                    .to_string(),
                );
                result.push('\n');
            }
        }
        bot.send_message(chat_id, result).await?;
    }

    Ok(())
}

/// Show filters for a specific category for removal
pub async fn show_category_filters_for_removal(
    bot: Bot,
    chat_id: ChatId,
    message_id: MessageId,
    storage: Arc<dyn CategoryStorageTrait>,
    category_name: String,
) -> ResponseResult<()> {
    let categories = storage.get_chat_categories(chat_id).await;

    if let Some(patterns) = categories.get(&category_name) {
        if patterns.is_empty() {
            bot.edit_message_text(
                chat_id,
                message_id,
                format!("📁 No filters in category `{}`\\. Use `/add_filter {} <pattern>` to add one\\.", escape(&category_name), escape(&category_name)),
            )
            .await?;
        } else {
            let text = format!(
                "🗑️ **Select filter to remove from `{}`:**\n\nClick a button to place the command in your input box\\.",
                escape(&category_name)
            );

            // Create buttons for each filter using switch_inline_query_current_chat
            // Show position number (0-indexed) along with the pattern
            let mut buttons: Vec<Vec<InlineKeyboardButton>> = patterns
                .iter()
                .enumerate()
                .map(|(index, pattern)| {
                    vec![InlineKeyboardButton::switch_inline_query_current_chat(
                        format!("{}. {}", index, pattern),
                        Command::RemoveFilter {
                            category: Some(category_name.clone()),
                            position: Some(index),
                        }
                        .to_string(),
                    )]
                })
                .collect();

            // Add a back button
            buttons.push(vec![InlineKeyboardButton::callback(
                "↩️ Back",
                CallbackData::CmdRemoveFilter,
            )]);

            let keyboard = InlineKeyboardMarkup::new(buttons);

            bot.edit_message_text(chat_id, message_id, text)
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
            bot.edit_message_reply_markup(chat_id, message_id)
                .reply_markup(keyboard)
                .await?;
        }
    }

    Ok(())
}

/// Show filters for a specific category for editing
pub async fn show_category_filters_for_editing(
    bot: Bot,
    chat_id: ChatId,
    message_id: MessageId,
    storage: Arc<dyn CategoryStorageTrait>,
    category_name: String,
) -> ResponseResult<()> {
    let categories = storage.get_chat_categories(chat_id).await;

    if let Some(patterns) = categories.get(&category_name) {
        if patterns.is_empty() {
            bot.edit_message_text(
                chat_id,
                message_id,
                format!("📁 No filters in category `{}`\\. Use `/add_filter {} <pattern>` to add one\\.", escape(&category_name), escape(&category_name)),
            )
            .await?;
        } else {
            let text = format!(
                "✏️ **Select filter to edit from `{}`:**\n\nClick a button to edit the filter\\. The existing pattern will be pre\\-filled for you to modify\\.",
                escape(&category_name)
            );

            // Create buttons for each filter using switch_inline_query_current_chat
            // Show position number (0-indexed) along with the pattern
            // Pre-fill the existing pattern in the input box
            let mut buttons: Vec<Vec<InlineKeyboardButton>> = patterns
                .iter()
                .enumerate()
                .map(|(index, pattern)| {
                    vec![InlineKeyboardButton::switch_inline_query_current_chat(
                        format!("{}. {}", index, pattern),
                        CommandEditFilter {
                            category: Some(category_name.clone()),
                            position: Some(index),
                            pattern: Some(pattern.clone()),
                        }
                        .to_command_string(true),
                    )]
                })
                .collect();

            // Add a back button
            buttons.push(vec![InlineKeyboardButton::callback(
                "↩️ Back",
                CallbackData::CmdEditFilter,
            )]);

            let keyboard = InlineKeyboardMarkup::new(buttons);

            bot.edit_message_text(chat_id, message_id, text)
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
            bot.edit_message_reply_markup(chat_id, message_id)
                .reply_markup(keyboard)
                .await?;
        }
    }

    Ok(())
}
