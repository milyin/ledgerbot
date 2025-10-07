use std::collections::HashMap;
use std::sync::Arc;
use teloxide::prelude::*;
use tokio::sync::Mutex;

use crate::commands::Command;
use crate::config::BATCH_TIMEOUT_SECONDS;

/// Batch processing state for a chat
#[derive(Clone)]
pub struct BatchState {
    pub messages_count: usize,
    pub records_count: usize,
    pub total_sum: f64,
    pub commands: Vec<Result<Command, String>>, // Store commands for deferred execution
}

/// Per-chat batch storage - each chat has its own batch state
pub type BatchStorage = Arc<Mutex<HashMap<ChatId, BatchState>>>;

/// Create a new batch storage
pub fn create_batch_storage() -> BatchStorage {
    Arc::new(Mutex::new(HashMap::new()))
}

/// Add expense data to batch and return whether this is the first message in the batch
pub async fn add_to_batch(
    batch_storage: &BatchStorage,
    chat_id: ChatId,
    message_count: usize,
    total: f64,
    commands: Vec<Result<Command, String>>,
) -> bool {
    let mut batch_guard = batch_storage.lock().await;
    match batch_guard.get_mut(&chat_id) {
        Some(state) => {
            // Update existing batch for this chat
            state.messages_count += 1;
            state.records_count += message_count;
            state.total_sum += total;
            state.commands.extend(commands);
            false
        }
        None => {
            // Start new batch for this chat
            batch_guard.insert(
                chat_id,
                BatchState {
                    messages_count: 1,
                    records_count: message_count,
                    total_sum: total,
                    commands,
                },
            );
            true
        }
    }
}

/// Send batch report after timeout and execute stored commands
pub async fn send_batch_report(
    bot: Bot,
    batch_storage: BatchStorage,
    target_chat_id: ChatId,
    storage: crate::storage::ExpenseStorage,
    category_storage: crate::storage::CategoryStorage,
    msg: Message,
) {
    // Wait for the timeout period
    tokio::time::sleep(tokio::time::Duration::from_secs(BATCH_TIMEOUT_SECONDS)).await;

    let batch_data = {
        let mut batch_guard = batch_storage.lock().await;
        // Remove and return the batch state if it exists
        batch_guard.remove(&target_chat_id)
    };

    if let Some(state) = batch_data {
        // Execute all stored commands
        for result in state.commands {
            match result {
                Ok(cmd) => {
                    let exec_result = execute_command(
                        bot.clone(),
                        msg.clone(),
                        storage.clone(),
                        category_storage.clone(),
                        cmd,
                    )
                    .await;
                    if let Err(e) = exec_result {
                        log::error!("Failed to execute batched command: {}", e);
                    }
                }
                Err(err_msg) => {
                    // Send error message to user
                    log::warn!("Parse error in batch for chat {}: {}", target_chat_id, err_msg);
                    if let Err(e) = bot
                        .send_message(target_chat_id, format!("❌ {}", err_msg))
                        .await
                    {
                        log::error!("Failed to send error message: {}", e);
                    }
                }
            }
        }

        let report = format!(
            "✅ **Batch Summary Report**\n\n\
            Messages processed: {}\n\
            Records parsed: {}\n\
            Total amount: {:.2}\n\n\
            Use `/list` or `/report` to see all expenses.",
            state.messages_count, state.records_count, state.total_sum
        );

        if let Err(e) = bot.send_message(target_chat_id, report).await {
            log::error!("Failed to send batch report: {}", e);
        }
    }
}

/// Execute a single command (helper function for batch processing)
async fn execute_command(
    bot: Bot,
    msg: Message,
    storage: crate::storage::ExpenseStorage,
    category_storage: crate::storage::CategoryStorage,
    cmd: Command,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match cmd {
        Command::Start => {
            crate::commands::start_command(bot.clone(), msg.clone()).await?;
        }
        Command::Help => {
            crate::commands::help_command(bot.clone(), msg.clone()).await?;
        }
        Command::Expense {
            date,
            description,
            amount,
        } => {
            crate::commands::expense_command(
                bot.clone(),
                msg.clone(),
                storage.clone(),
                date,
                description,
                amount,
            )
            .await?;
        }
        Command::List => {
            crate::commands::list_command(bot.clone(), msg.clone(), storage.clone()).await?;
        }
        Command::Report => {
            crate::commands::report_command(
                bot.clone(),
                msg.clone(),
                storage.clone(),
                category_storage.clone(),
            )
            .await?;
        }
        Command::Clear => {
            crate::commands::clear_command(bot.clone(), msg.clone(), storage.clone()).await?;
        }
        Command::ClearCategories => {
            crate::commands::clear_categories_command(bot.clone(), msg.clone(), category_storage.clone())
                .await?;
        }
        Command::AddCategory { name } => {
            crate::commands::category_command(
                bot.clone(),
                msg.clone(),
                category_storage.clone(),
                name,
            )
            .await?;
        }
        Command::Categories => {
            crate::commands::categories_command(bot.clone(), msg.clone(), category_storage.clone())
                .await?;
        }
        Command::AddFilter { category, pattern } => {
            crate::commands::add_filter_command(
                bot.clone(),
                msg.clone(),
                category_storage.clone(),
                category,
                pattern,
            )
            .await?;
        }
        Command::RemoveCategory { name } => {
            crate::commands::remove_category_command(
                bot.clone(),
                msg.clone(),
                category_storage.clone(),
                name,
            )
            .await?;
        }
        Command::RemoveFilter { category, pattern } => {
            crate::commands::remove_filter_command(
                bot.clone(),
                msg.clone(),
                category_storage.clone(),
                category,
                pattern,
            )
            .await?;
        }
    }
    Ok(())
}
