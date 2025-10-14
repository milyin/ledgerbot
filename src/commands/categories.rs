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

/// Add a category (name only)
pub async fn category_command(
    bot: Bot,
    msg: Message,
    storage: Arc<dyn CategoryStorageTrait>,
    name: Option<String>,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    // Check if name is provided
    match name {
        None => {
            // Show the add category menu instead
            let sent_msg = bot.send_message(chat_id, "➕ Add Category").await?;
            add_category_menu(bot, chat_id, sent_msg.id).await?;
        }
        Some(name) => match storage.add_category(chat_id, name.clone()).await {
            Ok(()) => {
                bot.send_message(
                    chat_id,
                    format!(
                        "✅ Category `{}` created\\. Use {} to add regex patterns\\.",
                        escape(&name),
                        escape(Command::ADD_FILTER)
                    ),
                )
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
            }
            Err(err_msg) => {
                bot.send_message(chat_id, format!("ℹ️ {}", escape(&err_msg)))
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                    .await?;
            }
        },
    }

    Ok(())
}

/// List all categories as executable commands
pub async fn categories_command(
    bot: Bot,
    msg: Message,
    storage: Arc<dyn CategoryStorageTrait>,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let categories = storage.get_chat_categories(chat_id).await;

    if categories.is_empty() {
        bot.send_message(
            chat_id,
            format!(
                "📂 No categories defined yet\\. Use {} to create one\\.",
                escape(
                    &Command::AddCategory {
                        name: Some("<name>".into())
                    }
                    .to_string()
                )
            ),
        )
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await?;
    } else {
        let mut result = String::new();

        // Sort categories for consistent output
        let mut sorted_categories: Vec<_> = categories.iter().collect();
        sorted_categories.sort_by(|a, b| a.0.cmp(b.0));

        for (name, patterns) in sorted_categories {
            // First create the category
            result.push_str(
                &Command::AddCategory {
                    name: Some(name.clone()),
                }
                .to_string(),
            );
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

/// Remove a category
pub async fn remove_category_command(
    bot: Bot,
    msg: Message,
    storage: Arc<dyn CategoryStorageTrait>,
    name: Option<String>,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    match name {
        None => {
            // Show the remove category menu instead
            let sent_msg = bot.send_message(chat_id, "🗑️ Remove Category").await?;
            remove_category_menu(bot, chat_id, sent_msg.id, storage).await?;
        }
        Some(name) => {
            let categories = storage.get_chat_categories(chat_id).await;

            // Check if category exists
            if !categories.contains_key(&name) {
                bot.send_message(
                    chat_id,
                    format!("❌ Category `{}` does not exist\\.", escape(&name)),
                )
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
                return Ok(());
            }

            // Remove the category
            storage.remove_category(chat_id, &name).await;
            bot.send_message(
                chat_id,
                format!("✅ Category `{}` removed\\.", escape(&name)),
            )
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
        }
    }

    Ok(())
}

/// Show category removal interface
pub async fn remove_category_menu(
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
            "📂 No categories to remove\\. Use `/add_category <name>` to create one first\\.",
        )
        .await?;
    } else {
        let text = "🗑️ **Select category to remove:**\n\nClick a button to place the command in your input box\\.";

        // Create buttons for each category using switch_inline_query_current_chat
        let buttons: Vec<Vec<InlineKeyboardButton>> = categories
            .keys()
            .map(|name| {
                vec![InlineKeyboardButton::switch_inline_query_current_chat(
                    format!("🗑️ {}", name),
                    Command::RemoveCategory {
                        name: Some(name.clone()),
                    }
                    .to_string(),
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

/// Show add category menu
pub async fn add_category_menu(
    bot: Bot,
    chat_id: ChatId,
    message_id: MessageId,
) -> ResponseResult<()> {
    let text = "➕ **Add a new category:**\n\nClick the button below and type the category name\\.";

    let keyboard = InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::switch_inline_query_current_chat(
            "➕ Add Category",
            format!("{} ", CommandAddCategory::NAME),
        ),
    ]]);

    bot.edit_message_text(chat_id, message_id, text)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await?;
    bot.edit_message_reply_markup(chat_id, message_id)
        .reply_markup(keyboard)
        .await?;

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
                        Command::EditFilter {
                            category: Some(category_name.clone()),
                            position: Some(index),
                            pattern: Some(pattern.clone()),
                        }
                        .to_string(),
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
