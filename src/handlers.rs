use teloxide::prelude::*;
use teloxide::types::CallbackQuery;

use crate::batch::{add_to_batch, send_batch_report, BatchStorage};
use crate::commands::{categories_command, clear_command, help_command, list_command, remove_category_menu};
use crate::parser::parse_expenses;
use crate::storage::{add_expenses, remove_category, CategoryStorage, ExpenseStorage};

/// Handle text messages containing potential expense data
pub async fn handle_text_message(
    bot: Bot,
    msg: Message,
    storage: ExpenseStorage,
    batch_storage: BatchStorage,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    if let Some(text) = msg.text() {
        // Parse expenses from the message
        let parsed_expenses = parse_expenses(text);

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
