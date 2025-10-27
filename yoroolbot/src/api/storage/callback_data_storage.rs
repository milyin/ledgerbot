use std::{collections::HashMap, sync::Arc};

use teloxide::types::ChatId;
use tokio::sync::Mutex;

/// Trait for callback data storage operations (maps short references to full callback data)
/// This is used to work around Telegram's 64-byte limit on callback data
#[async_trait::async_trait]
pub trait CallbackDataStorageTrait: Send + Sync {
    /// Store callback data and return a short reference string
    /// The reference is based on (message_id, button_position)
    async fn store_callback_data(
        &self,
        chat_id: ChatId,
        message_id: i32,
        button_pos: usize,
        data: String,
    ) -> String;

    /// Retrieve original callback data from a reference string
    async fn get_callback_data(&self, reference: &str) -> Option<String>;

    /// Clear all callback data for a specific message
    async fn clear_message_callbacks(&self, chat_id: ChatId, message_id: i32);
}

/// Callback data storage - maps (chat_id, message_id, button_pos) to full callback data
/// This is used to work around Telegram's 64-byte limit on callback data
#[derive(Clone)]
pub struct CallbackDataStorage {
    data: Arc<Mutex<HashMap<(ChatId, i32, usize), String>>>,
}

impl CallbackDataStorage {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for CallbackDataStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Implement CallbackDataStorageTrait for CallbackDataStorage
#[async_trait::async_trait]
impl CallbackDataStorageTrait for CallbackDataStorage {
    async fn store_callback_data(
        &self,
        chat_id: ChatId,
        message_id: i32,
        button_pos: usize,
        data: String,
    ) -> String {
        let mut storage_guard = self.data.lock().await;
        let key = (chat_id, message_id, button_pos);
        storage_guard.insert(key, data);
        // Return a compact reference string: "cb:{chat_id}:{message_id}:{button_pos}"
        format!("cb:{}:{}:{}", chat_id, message_id, button_pos)
    }

    async fn get_callback_data(&self, reference: &str) -> Option<String> {
        // Parse reference string: "cb:{chat_id}:{message_id}:{button_pos}"
        let parts: Vec<&str> = reference.split(':').collect();
        if parts.len() != 4 || parts[0] != "cb" {
            return None;
        }

        let chat_id = parts[1].parse::<i64>().ok()?;
        let message_id = parts[2].parse::<i32>().ok()?;
        let button_pos = parts[3].parse::<usize>().ok()?;

        let storage_guard = self.data.lock().await;
        storage_guard
            .get(&(ChatId(chat_id), message_id, button_pos))
            .cloned()
    }

    async fn clear_message_callbacks(&self, chat_id: ChatId, message_id: i32) {
        let mut storage_guard = self.data.lock().await;
        storage_guard.retain(|(cid, mid, _), _| *cid != chat_id || *mid != message_id);
    }
}
