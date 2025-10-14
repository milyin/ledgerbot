use std::sync::Arc;

use teloxide::prelude::*;
use yoroolbot::{markdown::MarkdownStringSendMessage, markdown_format};

use crate::{
    commands::{Command, execute_command},
    config::BATCH_TIMEOUT_SECONDS,
    storage_traits::{BatchStorageTrait, StorageTrait},
};

/// Add expense data to batch and return whether this is the first message in the batch
pub async fn add_to_batch(
    batch_storage: Arc<dyn BatchStorageTrait>,
    chat_id: ChatId,
    commands: Vec<Result<Command, String>>,
) -> bool {
    batch_storage.add_to_batch(chat_id, commands).await
}

/// Send batch report after timeout and execute stored commands
pub async fn execute_batch(
    bot: Bot,
    batch_storage: Arc<dyn BatchStorageTrait>,
    target_chat_id: ChatId,
    storage: Arc<dyn StorageTrait>,
    msg: Message,
) {
    // Wait for the timeout period
    tokio::time::sleep(tokio::time::Duration::from_secs(BATCH_TIMEOUT_SECONDS)).await;

    let batch_data = batch_storage.consume_batch(target_chat_id).await;

    let mut expense_count: usize = 0;
    let mut total_amount: f64 = 0.0;

    if let Some(state) = batch_data {
        // Execute all stored commands
        for result in state {
            match result {
                Ok(cmd) => {
                    if let Command::Expense {
                        amount: Some(amt_val),
                        ..
                    } = cmd
                    {
                        expense_count += 1;
                        total_amount += amt_val;
                    }
                    let exec_result =
                        execute_command(bot.clone(), msg.clone(), storage.clone(), cmd, true).await;
                    if let Err(e) = exec_result {
                        log::error!("Failed to execute batched command: {}", e);
                    }
                }
                Err(err_msg) => {
                    // Send error message to user
                    log::warn!(
                        "Parse error in batch for chat {}: {}",
                        target_chat_id,
                        err_msg
                    );
                    if let Err(e) = bot
                        .send_markdown_message(target_chat_id, markdown_format!("❌ {}", err_msg))
                        .await
                    {
                        log::error!("Failed to send error message: {}", e);
                    }
                }
            }
        }

        if let Err(e) = bot
            .send_markdown_message(
                target_chat_id,
                markdown_format!(
                    "✅ **Batch Summary Report**\n\n\
            Expense records parsed: {}\n\
            Total amount: {}\n\n\
            Use {} or {} to see all expenses\\.",
                    expense_count,
                    total_amount,
                    Command::LIST,
                    Command::REPORT
                ),
            )
            .await
        {
            log::error!("Failed to send batch report: {}", e);
        }
    }
}
