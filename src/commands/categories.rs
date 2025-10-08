use teloxide::{
    Bot,
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
    types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, Message, MessageId},
};

use crate::handlers::CallbackData;
use crate::storage::{CategoryStorage, add_category, get_chat_categories};

/// Add a category (name only)
pub async fn category_command(
    bot: Bot,
    msg: Message,
    category_storage: CategoryStorage,
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
        Some(name) => match add_category(&category_storage, chat_id, name.clone()).await {
            Ok(()) => {
                bot.send_message(
                    chat_id,
                    format!(
                        "✅ Category '{}' created. Use /add_filter to add regex patterns.",
                        name
                    ),
                )
                .await?;
            }
            Err(err_msg) => {
                bot.send_message(chat_id, format!("ℹ️ {}", err_msg)).await?;
            }
        },
    }

    Ok(())
}

/// List all categories as executable commands
pub async fn categories_command(
    bot: Bot,
    msg: Message,
    category_storage: CategoryStorage,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let categories = get_chat_categories(&category_storage, chat_id).await;

    if categories.is_empty() {
        bot.send_message(chat_id, "No categories defined yet.")
            .await?;
    } else {
        let mut result = String::new();

        // Sort categories for consistent output
        let mut sorted_categories: Vec<_> = categories.iter().collect();
        sorted_categories.sort_by(|a, b| a.0.cmp(b.0));

        for (name, patterns) in sorted_categories {
            // First create the category
            result.push_str(&format!("/add_category {}\n", name));

            // Then assign patterns if they exist
            for pattern in patterns {
                result.push_str(&format!("/add_filter {} {}\n", name, pattern));
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
    category_storage: CategoryStorage,
    name: Option<String>,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    match name {
        None => {
            // Show the remove category menu instead
            let sent_msg = bot.send_message(chat_id, "❌ Remove Category").await?;
            remove_category_menu(bot, chat_id, sent_msg.id, category_storage).await?;
        }
        Some(name) => {
            let categories = get_chat_categories(&category_storage, chat_id).await;

            // Check if category exists
            if !categories.contains_key(&name) {
                bot.send_message(chat_id, format!("❌ Category '{}' does not exist.", name))
                    .await?;
                return Ok(());
            }

            // Remove the category
            crate::storage::remove_category(&category_storage, chat_id, &name).await;
            bot.send_message(chat_id, format!("✅ Category '{}' removed.", name))
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
    category_storage: CategoryStorage,
) -> ResponseResult<()> {
    let categories = get_chat_categories(&category_storage, chat_id).await;

    if categories.is_empty() {
        bot.edit_message_text(chat_id, message_id, "No categories to remove.")
            .await?;
    } else {
        let text = "❌ **Select category to remove:**\n\nClick a button to place the command in your input box.";

        // Create buttons for each category using switch_inline_query_current_chat
        let buttons: Vec<Vec<InlineKeyboardButton>> = categories
            .keys()
            .map(|name| {
                vec![InlineKeyboardButton::switch_inline_query_current_chat(
                    format!("🚫 {}", name),
                    format!("/remove_category {}", name),
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

/// Show add category menu
pub async fn add_category_menu(
    bot: Bot,
    chat_id: ChatId,
    message_id: MessageId,
) -> ResponseResult<()> {
    let text = "➕ **Add a new category:**\n\nClick the button below and type the category name.";

    let keyboard = InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::switch_inline_query_current_chat("➕ Add Category", "/add_category "),
    ]]);

    bot.edit_message_text(chat_id, message_id, text).await?;
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
    category_storage: CategoryStorage,
    category_name: String,
) -> ResponseResult<()> {
    let categories = get_chat_categories(&category_storage, chat_id).await;

    if let Some(patterns) = categories.get(&category_name) {
        if patterns.is_empty() {
            bot.edit_message_text(
                chat_id,
                message_id,
                format!("No filters in category '{}'.", category_name),
            )
            .await?;
        } else {
            let text = format!(
                "�️ **Select filter to remove from '{}':**\n\nClick a button to place the command in your input box.",
                category_name
            );

            // Create buttons for each filter using switch_inline_query_current_chat
            let mut buttons: Vec<Vec<InlineKeyboardButton>> = patterns
                .iter()
                .map(|pattern| {
                    vec![InlineKeyboardButton::switch_inline_query_current_chat(
                        pattern.clone(),
                        format!("/remove_filter {} {}", category_name, pattern),
                    )]
                })
                .collect();

            // Add a back button
            buttons.push(vec![InlineKeyboardButton::callback(
                "↩️ Back",
                CallbackData::CmdRemoveFilter.to_callback_string(),
            )]);

            let keyboard = InlineKeyboardMarkup::new(buttons);

            bot.edit_message_text(chat_id, message_id, text).await?;
            bot.edit_message_reply_markup(chat_id, message_id)
                .reply_markup(keyboard)
                .await?;
        }
    }

    Ok(())
}
