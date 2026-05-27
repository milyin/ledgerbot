use std::sync::Arc;

use teloxide::types::ChatId;

use crate::commands::Command;
use crate::data_store::DataStoreTrait;

/// Type alias for batch data (list of command results)
pub type BatchData = Vec<Result<Command, String>>;

/// Trait for batch storage read operations (temporary command batching)
#[async_trait::async_trait]
pub trait BatchStorageReadTrait: Send + Sync {
    /// Get current batch data without consuming it
    async fn get_batch(&self) -> Option<BatchData>;
}

/// Trait for batch storage operations (temporary command batching)
#[async_trait::async_trait]
pub trait BatchStorageTrait: BatchStorageReadTrait + Send + Sync {
    /// Add commands to batch and return whether this is the first message in the batch
    async fn add_to_batch(&self, commands: Vec<Result<Command, String>>) -> bool;

    /// Consume and remove batch data
    async fn consume_batch(&self) -> Option<BatchData>;
}

/// Per-chat batch storage for temporary command batching during message processing
/// Uses a single key "batch" to store the command list
#[derive(Clone)]
pub struct BatchStorage {
    store: Arc<dyn DataStoreTrait<BatchData>>,
    chat_id: ChatId,
}

impl BatchStorage {
    /// Create a new BatchStorage with the given DataStore and chat ID
    pub fn new(store: Arc<dyn DataStoreTrait<BatchData>>, chat_id: ChatId) -> Self {
        Self { store, chat_id }
    }
}

/// Implement BatchStorageReadTrait for BatchStorage
#[async_trait::async_trait]
impl BatchStorageReadTrait for BatchStorage {
    async fn get_batch(&self) -> Option<BatchData> {
        self.store.get(self.chat_id, "batch").await
    }
}

/// Implement BatchStorageTrait for BatchStorage
#[async_trait::async_trait]
impl BatchStorageTrait for BatchStorage {
    async fn add_to_batch(&self, commands: Vec<Result<Command, String>>) -> bool {
        match self.store.get(self.chat_id, "batch").await {
            Some(mut existing_batch) => {
                // Update existing batch for this chat
                existing_batch.extend(commands);
                self.store.set(self.chat_id, "batch", existing_batch).await;
                false
            }
            None => {
                // Start new batch for this chat
                self.store.set(self.chat_id, "batch", commands).await;
                true
            }
        }
    }

    async fn consume_batch(&self) -> Option<BatchData> {
        let batch = self.store.get(self.chat_id, "batch").await;
        if batch.is_some() {
            self.store.remove(self.chat_id, "batch").await;
        }
        batch
    }
}
