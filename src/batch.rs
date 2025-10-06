use std::collections::HashMap;
use std::sync::Arc;
use teloxide::prelude::*;
use tokio::sync::Mutex;

use crate::config::BATCH_TIMEOUT_SECONDS;

/// Batch processing state for a chat
#[derive(Clone)]
pub struct BatchState {
    pub messages_count: usize,
    pub records_count: usize,
    pub total_sum: f64,
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
) -> bool {
    let mut batch_guard = batch_storage.lock().await;
    match batch_guard.get_mut(&chat_id) {
        Some(state) => {
            // Update existing batch for this chat
            state.messages_count += 1;
            state.records_count += message_count;
            state.total_sum += total;
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
                },
            );
            true
        }
    }
}

/// Send batch report after timeout
pub async fn send_batch_report(bot: Bot, batch_storage: BatchStorage, target_chat_id: ChatId) {
    // Wait for the timeout period
    tokio::time::sleep(tokio::time::Duration::from_secs(BATCH_TIMEOUT_SECONDS)).await;

    let batch_data = {
        let mut batch_guard = batch_storage.lock().await;
        // Remove and return the batch state if it exists
        batch_guard.remove(&target_chat_id)
    };

    if let Some(state) = batch_data {
        let report = format!(
            "📊 **Batch Summary Report**\n\n\
            📨 Messages processed: {}\n\
            📝 Records parsed: {}\n\
            💰 Total amount: {:.2}\n\n\
            Use `/list` or `/report` to see all expenses.",
            state.messages_count, state.records_count, state.total_sum
        );

        if let Err(e) = bot.send_message(target_chat_id, report).await {
            log::error!("Failed to send batch report: {}", e);
        }
    }
}
