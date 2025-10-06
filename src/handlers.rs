use teloxide::prelude::*;
use teloxide::types::CallbackQuery;
use teloxide::utils::command::BotCommands;

use crate::batch::{add_to_batch, send_batch_report, BatchStorage};
use crate::commands::{categories_command, clear_command, help_command, list_command, remove_category_menu, add_filter_menu, remove_filter_menu, show_category_filters_for_removal, show_filter_word_suggestions, Command};
use crate::parser::parse_expenses;
use crate::storage::{add_expenses, remove_category, remove_category_filter, get_filter_selection, set_filter_selection, clear_filter_selection, CategoryStorage, ExpenseStorage, FilterSelectionStorage};

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
        
        // Parse expenses and commands from the message, with bot name filtering
        let (parsed_expenses, parsed_commands) = parse_expenses(text, bot_name.as_deref());

        // Execute parsed commands
        for command_str in parsed_commands {
            if let Ok(cmd) = Command::parse(&command_str, bot_name.as_deref().unwrap_or("")) {
                // Execute the command
                match cmd {
                    Command::Help | Command::Start => {
                        help_command(bot.clone(), msg.clone()).await?;
                    }
                    Command::List => {
                        list_command(bot.clone(), msg.clone(), storage.clone(), category_storage.clone()).await?;
                    }
                    Command::Clear => {
                        clear_command(bot.clone(), msg.clone(), storage.clone()).await?;
                    }
                    Command::Category { name } => {
                        crate::commands::category_command(bot.clone(), msg.clone(), category_storage.clone(), name).await?;
                    }
                    Command::Categories => {
                        categories_command(bot.clone(), msg.clone(), category_storage.clone()).await?;
                    }
                    Command::AddFilter { category, pattern } => {
                        crate::commands::add_filter_command(bot.clone(), msg.clone(), category_storage.clone(), category, pattern).await?;
                    }
                }
            }
        }

        if !parsed_expenses.is_empty() {
            // Store the expenses in chat-specific storage
            add_expenses(&storage, chat_id, parsed_expenses.clone()).await;

            // Update batch state for this chat
            let total_parsed: f64 = parsed_expenses.iter().map(|(_, amount)| amount).sum();
            let is_first_message =
                add_to_batch(&batch_storage, chat_id, parsed_expenses.len(), total_parsed).await;

            // Start timeout task only for the first message in batch
            if is_first_message {
                let batch_clone = batch_storage.clone();
                let bot_clone = bot.clone();
                tokio::spawn(async move {
                    send_batch_report(bot_clone, batch_clone, chat_id).await;
                });
            }
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
) -> ResponseResult<()> {
    if let Some(data) = &q.data {
        // Answer the callback query to remove the loading state
        bot.answer_callback_query(q.id.clone()).await?;
        
        // Get the message that contained the button
        if let Some(message) = q.message {
            if let Some(msg) = message.regular_message() {
                let msg = msg.clone();
                let chat_id = msg.chat.id;
                
                // Handle remove_cat:CategoryName format
                if data.starts_with("remove_cat:") {
                    let category_name = data.strip_prefix("remove_cat:").unwrap();
                    remove_category(&category_storage, chat_id, category_name).await;
                    bot.send_message(chat_id, format!("✅ Category '{}' removed.", category_name))
                        .await?;
                    // Show the updated remove menu
                    remove_category_menu(bot, msg, category_storage).await?;
                } else if data.starts_with("add_filter_cat:") {
                    // Show word suggestions for a specific category
                    let category_name = data.strip_prefix("add_filter_cat:").unwrap().to_string();
                    // Clear any previous selection
                    clear_filter_selection(&filter_selection_storage, chat_id, &category_name).await;
                    show_filter_word_suggestions(bot, chat_id, storage.clone(), category_storage, filter_selection_storage.clone(), category_name).await?;
                } else if data.starts_with("toggle_word:") {
                    // Handle toggle_word:CategoryName:Word format
                    let parts: Vec<&str> = data.strip_prefix("toggle_word:").unwrap().splitn(2, ':').collect();
                    if parts.len() == 2 {
                        let category_name = parts[0].to_string();
                        let word = parts[1].to_string();
                        
                        // Get current selection
                        let mut selected_words = get_filter_selection(&filter_selection_storage, chat_id, &category_name).await;
                        
                        // Toggle the word
                        if let Some(pos) = selected_words.iter().position(|w| w == &word) {
                            selected_words.remove(pos);
                        } else {
                            selected_words.push(word);
                        }
                        
                        // Save updated selection
                        set_filter_selection(&filter_selection_storage, chat_id, category_name.clone(), selected_words).await;
                        
                        // Update the message with new selection
                        show_filter_word_suggestions(
                            bot, chat_id, storage.clone(), category_storage, filter_selection_storage.clone(), category_name
                        ).await?;
                    }
                } else if data.starts_with("apply_words:") {
                    // Handle apply_words:CategoryName format
                    let category_name = data.strip_prefix("apply_words:").unwrap().to_string();
                    
                    // Get selected words
                    let selected_words = get_filter_selection(&filter_selection_storage, chat_id, &category_name).await;
                    
                    if !selected_words.is_empty() {
                        let words = selected_words.join("|");
                        
                        // Clear the selection
                        clear_filter_selection(&filter_selection_storage, chat_id, &category_name).await;
                        
                        // Call add_filter_command with the combined words
                        crate::commands::add_filter_command(
                            bot.clone(), msg.clone(), category_storage.clone(), category_name, words
                        ).await?;
                    }
                } else if data.starts_with("remove_filter_cat:") {
                    // Show filters for a specific category
                    let category_name = data.strip_prefix("remove_filter_cat:").unwrap().to_string();
                    show_category_filters_for_removal(bot, chat_id, category_storage, category_name).await?;
                } else if data.starts_with("remove_filter:") {
                    // Handle remove_filter:CategoryName:Pattern format
                    let parts: Vec<&str> = data.strip_prefix("remove_filter:").unwrap().splitn(2, ':').collect();
                    if parts.len() == 2 {
                        let category_name = parts[0];
                        let pattern = parts[1];
                        remove_category_filter(&category_storage, chat_id, category_name, pattern).await;
                        bot.send_message(
                            chat_id,
                            format!("✅ Filter '{}' removed from category '{}'.", pattern, category_name)
                        )
                        .await?;
                        // Show the updated filters for this category
                        show_category_filters_for_removal(bot, chat_id, category_storage, category_name.to_string()).await?;
                    }
                } else {
                    match data.as_str() {
                        "cmd_list" => {
                            list_command(bot, msg, storage, category_storage).await?;
                        }
                        "cmd_clear" => {
                            clear_command(bot, msg, storage).await?;
                        }
                        "cmd_categories" => {
                            categories_command(bot, msg, category_storage).await?;
                        }
                        "cmd_remove_category" => {
                            remove_category_menu(bot, msg, category_storage).await?;
                        }
                        "cmd_add_filter" => {
                            add_filter_menu(bot, msg, category_storage).await?;
                        }
                        "cmd_remove_filter" => {
                            remove_filter_menu(bot, msg, category_storage).await?;
                        }
                        "cmd_back_to_help" => {
                            help_command(bot, msg).await?;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    
    Ok(())
}
