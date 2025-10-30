use std::{collections::HashMap, sync::Arc};

use teloxide::types::ChatId;
use tokio::sync::Mutex;

use crate::commands::Command;

/// Trait for batch storage operations (temporary command batching)
#[async_trait::async_trait]
pub trait BatchStorageTrait: Send + Sync {
    /// Add commands to batch and return whether this is the first message in the batch
    async fn add_to_batch(&self, chat_id: ChatId, commands: Vec<Result<Command, String>>) -> bool;

    /// Consume and remove batch data for a chat
    async fn consume_batch(&self, chat_id: ChatId) -> Option<Vec<Result<Command, String>>>;
}

type BatchStorageData = Arc<Mutex<HashMap<ChatId, Vec<Result<Command, String>>>>>;


/// Per-chat batch storage for temporary command batching during message processing
#[derive(Clone)]
pub struct BatchStorage {
    data: BatchStorageData,
}

impl BatchStorage {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

/// Implement BatchStorageTrait for BatchStorage
#[async_trait::async_trait]
impl BatchStorageTrait for BatchStorage {
    async fn add_to_batch(&self, chat_id: ChatId, commands: Vec<Result<Command, String>>) -> bool {
        let mut storage_guard = self.data.lock().await;
        match storage_guard.get_mut(&chat_id) {
            Some(state) => {
                // Update existing batch for this chat
                state.extend(commands);
                false
            }
            None => {
                // Start new batch for this chat
                storage_guard.insert(chat_id, commands);
                true
            }
        }
    }

    async fn consume_batch(&self, chat_id: ChatId) -> Option<Vec<Result<Command, String>>> {
        let mut storage_guard = self.data.lock().await;
        storage_guard.remove(&chat_id)
    }
}
