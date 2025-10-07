use std::collections::HashMap;
use std::sync::Arc;
use teloxide::prelude::*;
use tokio::sync::Mutex;

use crate::commands::Command;
use crate::config::BATCH_TIMEOUT_SECONDS;

/// Per-chat batch storage - each chat has its own batch state
pub type BatchStorage = Arc<Mutex<HashMap<ChatId, Vec<Result<Command, String>>>>>;

/// Create a new batch storage
pub fn create_batch_storage() -> BatchStorage {
    Arc::new(Mutex::new(HashMap::new()))
}

/// Add expense data to batch and return whether this is the first message in the batch
pub async fn add_to_batch(
    batch_storage: &BatchStorage,
    chat_id: ChatId,
    commands: Vec<Result<Command, String>>,
) -> bool {
    let mut batch_guard = batch_storage.lock().await;
    match batch_guard.get_mut(&chat_id) {
        Some(state) => {
            // Update existing batch for this chat
            state.extend(commands);
            false
        }
        None => {
            // Start new batch for this chat
            batch_guard.insert(
                chat_id,
                Vec::new(),
            );
            true
        }
    }
}

/// Send batch report after timeout and execute stored commands
pub async fn execute_batch(
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

    let mut expense_count: usize = 0;
    let mut total_amount: f64 = 0.0;

    if let Some(state) = batch_data {
        // Execute all stored commands
        for result in state {
            match result {
                Ok(cmd) => {
                    if let Command::Expense { ref amount, .. } = cmd {
                        if let Some(amt_str) = amount {
                            if let Ok(amt_val) = amt_str.parse::<f64>() {
                                expense_count += 1;
                                total_amount += amt_val;
                            }
                        }
                    }
                    let exec_result = execute_command(
                        bot.clone(),
                        msg.clone(),
                        storage.clone(),
                        category_storage.clone(),
                        cmd,
                        true
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
            Expense records parsed: {}\n\
            Total amount: {:.2}\n\n\
            Use `/list` or `/report` to see all expenses.",
            expense_count, total_amount
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
    silent: bool
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
                silent
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
