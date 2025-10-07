use std::collections::HashMap;
use std::sync::Arc;
use teloxide::prelude::*;
use tokio::sync::Mutex;

use crate::commands::{Command, execute_command};
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
