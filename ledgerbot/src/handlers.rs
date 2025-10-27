use std::{str::FromStr, sync::Arc};

use teloxide::{prelude::*, types::CallbackQuery, utils::command::BotCommands};
use yoroolbot::{markdown::MarkdownStringMessage, markdown_format};

use crate::{
    batch::{add_to_batch, execute_batch},
    commands::{Command, execute_command, filters::add_filter_menu, show_filter_word_suggestions},
    menus::common::unpack_callback_data,
    storage_traits::StorageTrait,
    utils::parse_expenses::parse_expenses,
};

/// Represents all possible callback data from inline keyboard buttons
#[derive(Debug, Clone, PartialEq)]
pub enum CallbackData {
    /// Show filter word suggestions for a category
    AddFilterCategory(String),
    /// Toggle a word in filter selection
    ToggleWord { category: String, word: String },
    /// Navigate to previous page
    PagePrev(String),
    /// Navigate to next page
    PageNext(String),
    /// Command: Add filter menu
    CmdAddFilter,
    /// No operation (inactive button)
    Noop,
}

impl FromStr for CallbackData {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(category) = s.strip_prefix("add_filter_cat:") {
            Ok(CallbackData::AddFilterCategory(category.to_string()))
        } else if let Some(rest) = s.strip_prefix("toggle_word:") {
            let parts: Vec<&str> = rest.splitn(2, ':').collect();
            if parts.len() == 2 {
                Ok(CallbackData::ToggleWord {
                    category: parts[0].to_string(),
                    word: parts[1].to_string(),
                })
            } else {
                Err(format!("Invalid toggle_word format: {}", s))
            }
        } else if let Some(category) = s.strip_prefix("page_prev:") {
            Ok(CallbackData::PagePrev(category.to_string()))
        } else if let Some(category) = s.strip_prefix("page_next:") {
            Ok(CallbackData::PageNext(category.to_string()))
        } else {
            match s {
                "cmd_add_filter" => Ok(CallbackData::CmdAddFilter),
                "noop" => Ok(CallbackData::Noop),
                _ => Err(format!("Unknown callback data: {}", s)),
            }
        }
    }
}

impl From<CallbackData> for String {
    fn from(data: CallbackData) -> String {
        match data {
            CallbackData::AddFilterCategory(cat) => format!("add_filter_cat:{}", cat),
            CallbackData::ToggleWord { category, word } => {
                format!("toggle_word:{}:{}", category, word)
            }
            CallbackData::PagePrev(cat) => format!("page_prev:{}", cat),
            CallbackData::PageNext(cat) => format!("page_next:{}", cat),
            CallbackData::CmdAddFilter => "cmd_add_filter".to_string(),
            CallbackData::Noop => "noop".to_string(),
        }
    }
}

/// Handle text messages containing potential expense data
pub async fn handle_text_message(
    bot: Bot,
    msg: Message,
    storage: Arc<dyn StorageTrait>,
) -> ResponseResult<()> {
    if let Some(text) = msg.text() {
        // Get bot username for filtering
        let bot_name = bot.get_me().await.ok().map(|me| me.username().to_string());

        // Get message timestamp (Unix timestamp in seconds)
        // Use forward_date if available (for forwarded messages), otherwise use msg.date
        let timestamp = msg.forward_date().unwrap_or(msg.date).timestamp();

        // Parse commands from the message, with bot name filtering and timestamp
        // Text expenses are now converted to Command::Expense variants
        let parsed_results = parse_expenses(text, bot_name.as_deref(), timestamp);

        log::info!(
            "Parsed {} results from chat {}",
            parsed_results.len(),
            msg.chat.id
        );

        // Check if we should process this message in batch mode
        let is_multiline = text.lines().filter(|line| !line.trim().is_empty()).count() > 1;
        let is_forwarded = msg.forward_date().is_some();

        // For multiline or forwarded messages, collect commands for batch execution.
        // For single-line, non-forwarded messages, execute immediately.
        if is_multiline || is_forwarded {
            // Add to batch storage for deferred execution
            let batch_storage = storage.clone().as_batch_storage();
            let is_first_message =
                add_to_batch(batch_storage.clone(), msg.chat.clone(), parsed_results).await;

            // Start timeout task only for the first message in batch
            if is_first_message {
                let bot_clone = bot.clone();
                let storage_clone = storage.clone();
                let msg_clone = msg.clone();
                tokio::spawn(async move {
                    execute_batch(
                        bot_clone,
                        batch_storage,
                        msg_clone.chat.clone(),
                        storage_clone,
                        msg_clone,
                    )
                    .await;
                });
            }
        } else {
            // Single-line message: execute immediately (existing behavior)
            for result in parsed_results {
                match result {
                    Ok(cmd) => {
                        // Execute the command using the shared execute_command function
                        let exec_result = execute_command(
                            bot.clone(),
                            msg.chat.clone(),
                            None,
                            msg.clone(),
                            storage.clone(),
                            cmd,
                            false,
                        )
                        .await;
                        if let Err(e) = exec_result {
                            log::error!("Failed to execute command: {}", e);
                            bot.markdown_message(
                                msg.chat.id,
                                None,
                                markdown_format!("❌ Error: {}", e.to_string()),
                            )
                            .await?;
                        }
                    }
                    Err(err_msg) => {
                        // Send error message to user
                        log::warn!("Parse error in chat {}: {}", msg.chat.id, err_msg);
                        bot.send_markdown_message(msg.chat.id, markdown_format!("❌ {}", err_msg))
                            .await?;
                    }
                }
            }

            // For single-line messages with expenses, we don't batch - already executed above
        }
    }

    Ok(())
}

/// Handle callback queries from inline keyboard buttons
pub async fn handle_callback_query(
    bot: Bot,
    q: CallbackQuery,
    storage: Arc<dyn StorageTrait>,
) -> ResponseResult<()> {
    let bot_username = bot.get_me().await?.username().to_string();
    // Answer the callback query to remove the loading state
    bot.answer_callback_query(q.id.clone()).await?;

    // Get the message that contained the button
    let Some(message) = q.message else {
        return Ok(());
    };

    let Some(msg) = message.regular_message() else {
        return Ok(());
    };

    let msg = msg.clone();
    let chat_id = msg.chat.id;

    // Parse callback data string into enum
    let Some(data_str) = &q.data else {
        return Ok(());
    };

    log::info!("Received callback data: {}", data_str);

    // Unpack callback data from storage if needed
    let callback_storage = storage.clone().as_callback_data_storage();
    let unpacked_data = unpack_callback_data(&callback_storage, data_str).await;

    log::info!("Unpacked callback data: {}", unpacked_data);

    // Try to parse the callback data as command
    if let Ok(cmd) = Command::parse(&unpacked_data, &bot_username) {
        log::info!("Parsed command from callback: {:?}", cmd);
        // Execute the command using the shared execute_command function
        if let Err(e) = execute_command(
            bot.clone(),
            msg.chat.clone(),
            Some(msg.id),
            msg.clone(),
            storage.clone(),
            cmd.clone(),
            false,
        )
        .await
        {
            log::error!("Failed to execute command from callback: {}", e);
            bot.send_markdown_message(
                chat_id,
                markdown_format!(
                    "❌ Error executing command `{}`: {}",
                    cmd.to_string(),
                    e.to_string()
                ),
            )
            .await?;
        }
        return Ok(());
    }

    let callback_data = match CallbackData::from_str(&unpacked_data) {
        Ok(data) => data,
        Err(err) => {
            log::warn!("Invalid callback data '{}': {}", unpacked_data, err);
            return Ok(());
        }
    };

    // Handle the callback using pattern matching
    match callback_data {
        CallbackData::AddFilterCategory(category_name) => {
            // Clear any previous selection and page offset
            storage
                .clone()
                .as_filter_selection_storage()
                .clear_filter_selection(chat_id, &category_name)
                .await;
            storage
                .clone()
                .as_filter_page_storage()
                .clear_filter_page_offset(chat_id, &category_name)
                .await;
            show_filter_word_suggestions(
                bot,
                chat_id,
                message.id(),
                storage.clone(),
                category_name,
            )
            .await?;
        }

        CallbackData::ToggleWord { category, word } => {
            // Get current selection
            let mut selected_words = storage
                .clone()
                .as_filter_selection_storage()
                .get_filter_selection(chat_id, &category)
                .await;

            // Toggle the word
            if let Some(pos) = selected_words.iter().position(|w| w == &word) {
                selected_words.remove(pos);
            } else {
                selected_words.push(word);
            }

            // Save updated selection
            storage
                .clone()
                .as_filter_selection_storage()
                .set_filter_selection(chat_id, category.clone(), selected_words)
                .await;

            // Update the message with new selection
            show_filter_word_suggestions(bot, chat_id, message.id(), storage.clone(), category)
                .await?;
        }

        CallbackData::PagePrev(category_name) => {
            // Get current page offset and decrease by 20
            let current_offset = storage
                .clone()
                .as_filter_page_storage()
                .get_filter_page_offset(chat_id, &category_name)
                .await;
            let new_offset = current_offset.saturating_sub(20);

            // Update page offset
            storage
                .clone()
                .as_filter_page_storage()
                .set_filter_page_offset(chat_id, category_name.clone(), new_offset)
                .await;

            // Refresh the display
            show_filter_word_suggestions(
                bot,
                chat_id,
                message.id(),
                storage.clone(),
                category_name,
            )
            .await?;
        }

        CallbackData::PageNext(category_name) => {
            // Get current page offset and increase by 20
            let current_offset = storage
                .clone()
                .as_filter_page_storage()
                .get_filter_page_offset(chat_id, &category_name)
                .await;
            let new_offset = current_offset + 20;

            // Update page offset
            storage
                .clone()
                .as_filter_page_storage()
                .set_filter_page_offset(chat_id, category_name.clone(), new_offset)
                .await;

            // Refresh the display
            show_filter_word_suggestions(
                bot,
                chat_id,
                message.id(),
                storage.clone(),
                category_name,
            )
            .await?;
        }

        CallbackData::CmdAddFilter => {
            add_filter_menu(
                bot,
                chat_id,
                message.id(),
                storage.clone().as_category_storage(),
            )
            .await?;
        }

        CallbackData::Noop => {
            // Inactive button - do nothing
        }
    }

    Ok(())
}
