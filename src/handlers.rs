use teloxide::prelude::*;
use teloxide::types::CallbackQuery;

use crate::batch::{BatchStorage, add_to_batch, execute_batch};
use crate::commands::categories::{
    categories_command, remove_category_menu, show_category_filters_for_removal,
};
use crate::commands::expenses::{clear_command, list_command};
use crate::commands::filters::{add_filter_command, add_filter_menu, remove_filter_menu};
use crate::commands::help::help_command;
use crate::commands::report::report_command;
use crate::commands::{execute_command, show_filter_word_suggestions};
use crate::parser::parse_expenses;
use crate::storage::{
    CategoryStorage, ExpenseStorage, FilterPageStorage, FilterSelectionStorage,
    clear_filter_page_offset, clear_filter_selection, get_filter_page_offset, get_filter_selection,
    remove_category, remove_category_filter, set_filter_page_offset, set_filter_selection,
};

/// Handle text messages containing potential expense data
pub async fn handle_text_message(
    bot: Bot,
    msg: Message,
    storage: ExpenseStorage,
    batch_storage: BatchStorage,
    category_storage: CategoryStorage,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

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
            chat_id
        );

        // Check if we should process this message in batch mode
        let is_multiline = text.lines().filter(|line| !line.trim().is_empty()).count() > 1;
        let is_forwarded = msg.forward_date().is_some();

        // For multiline or forwarded messages, collect commands for batch execution.
        // For single-line, non-forwarded messages, execute immediately.
        if is_multiline || is_forwarded {
            // Add to batch storage for deferred execution
            let is_first_message = add_to_batch(&batch_storage, chat_id, parsed_results).await;

            // Start timeout task only for the first message in batch
            if is_first_message {
                let batch_clone = batch_storage.clone();
                let bot_clone = bot.clone();
                let storage_clone = storage.clone();
                let category_storage_clone = category_storage.clone();
                let msg_clone = msg.clone();
                tokio::spawn(async move {
                    execute_batch(
                        bot_clone,
                        batch_clone,
                        chat_id,
                        storage_clone,
                        category_storage_clone,
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
                            msg.clone(),
                            storage.clone(),
                            category_storage.clone(),
                            cmd,
                            false,
                        )
                        .await;
                        if let Err(e) = exec_result {
                            log::error!("Failed to execute command: {}", e);
                            bot.send_message(chat_id, format!("❌ Error: {}", e))
                                .await?;
                        }
                    }
                    Err(err_msg) => {
                        // Send error message to user
                        log::warn!("Parse error in chat {}: {}", chat_id, err_msg);
                        bot.send_message(chat_id, format!("❌ {}", err_msg)).await?;
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
    storage: ExpenseStorage,
    category_storage: CategoryStorage,
    filter_selection_storage: FilterSelectionStorage,
    filter_page_storage: FilterPageStorage,
) -> ResponseResult<()> {
    if let Some(data) = &q.data {
        // Answer the callback query to remove the loading state
        bot.answer_callback_query(q.id.clone()).await?;

        // Get the message that contained the button
        if let Some(message) = q.message
            && let Some(msg) = message.regular_message()
        {
            let msg = msg.clone();
            let chat_id = msg.chat.id;

            // Handle remove_cat:CategoryName format
            if data.starts_with("remove_cat:") {
                let category_name = data.strip_prefix("remove_cat:").unwrap();
                remove_category(&category_storage, chat_id, category_name).await;
                // Show the updated remove menu
                remove_category_menu(bot, chat_id, message.id(), category_storage).await?;
            } else if data.starts_with("add_filter_cat:") {
                // Show word suggestions for a specific category
                let category_name = data.strip_prefix("add_filter_cat:").unwrap().to_string();
                // Clear any previous selection and page offset
                clear_filter_selection(&filter_selection_storage, chat_id, &category_name).await;
                clear_filter_page_offset(&filter_page_storage, chat_id, &category_name).await;
                show_filter_word_suggestions(
                    bot,
                    chat_id,
                    message.id(),
                    storage.clone(),
                    category_storage,
                    filter_selection_storage.clone(),
                    filter_page_storage.clone(),
                    category_name,
                )
                .await?;
            } else if data.starts_with("toggle_word:") {
                // Handle toggle_word:CategoryName:Word format
                let parts: Vec<&str> = data
                    .strip_prefix("toggle_word:")
                    .unwrap()
                    .splitn(2, ':')
                    .collect();
                if parts.len() == 2 {
                    let category_name = parts[0].to_string();
                    let word = parts[1].to_string();

                    // Get current selection
                    let mut selected_words =
                        get_filter_selection(&filter_selection_storage, chat_id, &category_name)
                            .await;

                    // Toggle the word
                    if let Some(pos) = selected_words.iter().position(|w| w == &word) {
                        selected_words.remove(pos);
                    } else {
                        selected_words.push(word);
                    }

                    // Save updated selection
                    set_filter_selection(
                        &filter_selection_storage,
                        chat_id,
                        category_name.clone(),
                        selected_words,
                    )
                    .await;

                    // Update the message with new selection
                    show_filter_word_suggestions(
                        bot,
                        chat_id,
                        message.id(),
                        storage.clone(),
                        category_storage,
                        filter_selection_storage.clone(),
                        filter_page_storage.clone(),
                        category_name,
                    )
                    .await?;
                }
            } else if data.starts_with("page_prev:") {
                // Handle page_prev:CategoryName format
                let category_name = data.strip_prefix("page_prev:").unwrap().to_string();

                // Get current page offset and decrease by 20
                let current_offset =
                    get_filter_page_offset(&filter_page_storage, chat_id, &category_name).await;
                let new_offset = current_offset.saturating_sub(20);

                // Update page offset
                set_filter_page_offset(
                    &filter_page_storage,
                    chat_id,
                    category_name.clone(),
                    new_offset,
                )
                .await;

                // Refresh the display
                show_filter_word_suggestions(
                    bot,
                    chat_id,
                    message.id(),
                    storage.clone(),
                    category_storage,
                    filter_selection_storage.clone(),
                    filter_page_storage.clone(),
                    category_name,
                )
                .await?;
            } else if data.starts_with("page_next:") {
                // Handle page_next:CategoryName format
                let category_name = data.strip_prefix("page_next:").unwrap().to_string();

                // Get current page offset and increase by 20
                let current_offset =
                    get_filter_page_offset(&filter_page_storage, chat_id, &category_name).await;
                let new_offset = current_offset + 20;

                // Update page offset
                set_filter_page_offset(
                    &filter_page_storage,
                    chat_id,
                    category_name.clone(),
                    new_offset,
                )
                .await;

                // Refresh the display
                show_filter_word_suggestions(
                    bot,
                    chat_id,
                    message.id(),
                    storage.clone(),
                    category_storage,
                    filter_selection_storage.clone(),
                    filter_page_storage.clone(),
                    category_name,
                )
                .await?;
            } else if data.starts_with("apply_words:") {
                // Handle apply_words:CategoryName format
                let category_name = data.strip_prefix("apply_words:").unwrap().to_string();

                // Get selected words
                let selected_words =
                    get_filter_selection(&filter_selection_storage, chat_id, &category_name).await;

                if !selected_words.is_empty() {
                    // Escape each word and combine with case-insensitive OR pattern
                    let escaped_words: Vec<String> =
                        selected_words.iter().map(|w| regex::escape(w)).collect();
                    let pattern = format!("(?i)({})", escaped_words.join("|"));

                    // Clear the selection and page offset
                    clear_filter_selection(&filter_selection_storage, chat_id, &category_name)
                        .await;
                    clear_filter_page_offset(&filter_page_storage, chat_id, &category_name).await;

                    // Call add_filter_command with the combined pattern
                    add_filter_command(
                        bot.clone(),
                        msg.clone(),
                        category_storage.clone(),
                        Some(category_name),
                        Some(pattern),
                    )
                    .await?;
                }
            } else if data.starts_with("remove_filter_cat:") {
                // Show filters for a specific category
                let category_name = data.strip_prefix("remove_filter_cat:").unwrap().to_string();
                show_category_filters_for_removal(
                    bot,
                    chat_id,
                    message.id(),
                    category_storage,
                    category_name,
                )
                .await?;
            } else if data.starts_with("remove_filter:") {
                // Handle remove_filter:CategoryName:Pattern format
                let parts: Vec<&str> = data
                    .strip_prefix("remove_filter:")
                    .unwrap()
                    .splitn(2, ':')
                    .collect();
                if parts.len() == 2 {
                    let category_name = parts[0];
                    let pattern = parts[1];
                    remove_category_filter(&category_storage, chat_id, category_name, pattern)
                        .await;
                    // Show the updated filters for this category
                    show_category_filters_for_removal(
                        bot,
                        chat_id,
                        message.id(),
                        category_storage,
                        category_name.to_string(),
                    )
                    .await?;
                }
            } else {
                match data.as_str() {
                    "cmd_list" => {
                        list_command(bot, msg, storage).await?;
                    }
                    "cmd_report" => {
                        report_command(bot, msg, storage, category_storage).await?;
                    }
                    "cmd_clear" => {
                        clear_command(bot, msg, storage).await?;
                    }
                    "cmd_categories" => {
                        categories_command(bot, msg, category_storage).await?;
                    }
                    "cmd_remove_category" => {
                        remove_category_menu(bot, chat_id, message.id(), category_storage).await?;
                    }
                    "cmd_add_filter" => {
                        add_filter_menu(bot, chat_id, message.id(), category_storage).await?;
                    }
                    "cmd_remove_filter" => {
                        remove_filter_menu(bot, chat_id, message.id(), category_storage).await?;
                    }
                    "cmd_back_to_help" => {
                        help_command(bot, msg).await?;
                    }
                    "noop" => {
                        // Inactive button - do nothing
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
