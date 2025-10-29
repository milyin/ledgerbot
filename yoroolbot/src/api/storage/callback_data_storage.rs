use std::{collections::HashMap, sync::Arc};

use teloxide::types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup};
use tokio::sync::Mutex;

/// Represents different types of inline keyboard buttons
#[derive(Clone)]
pub enum ButtonData {
    /// Callback button with label and callback data
    Callback(String, String),
    /// Switch inline query button with label and query text
    SwitchInlineQuery(String, String),
}

impl From<(String, String)> for ButtonData {
    fn from((label, data): (String, String)) -> Self {
        ButtonData::Callback(label, data)
    }
}

impl From<(&str, &str)> for ButtonData {
    fn from((label, data): (&str, &str)) -> Self {
        ButtonData::Callback(label.to_string(), data.to_string())
    }
}

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

/// Pack callback data into an InlineKeyboardMarkup, storing long data in storage
/// and replacing it with short references.
///
/// This function takes rows of button data where each row contains ButtonData enum values.
/// For callback buttons, if the callback_data is longer than 64 bytes or contains non-ASCII
/// characters, it stores the data in CallbackDataStorage and replaces it with a short reference.
/// For switch inline query buttons, the query text is used directly without storage.
///
/// **Important:** This function clears any previously stored callback data for this message
/// to prevent memory leaks when updating message buttons.
///
/// # Arguments
/// * `storage` - The callback data storage trait
/// * `chat_id` - The chat ID for this message
/// * `message_id` - The message ID where buttons will be attached
/// * `rows` - Iterator of button rows, each row is an iterator of ButtonData values
pub async fn pack_callback_data<R, B>(
    storage: &Arc<dyn CallbackDataStorageTrait>,
    chat_id: ChatId,
    message_id: i32,
    rows: impl IntoIterator<Item = R>,
) -> InlineKeyboardMarkup
where
    R: IntoIterator<Item = B>,
    B: Into<ButtonData>,
{
    // Clear old callback data for this message to prevent memory leaks
    storage.clear_message_callbacks(chat_id, message_id).await;

    let mut button_rows = Vec::new();
    let mut button_pos = 0;

    for row in rows {
        let mut button_row = Vec::new();
        for item in row {
            let button_data: ButtonData = item.into();

            match button_data {
                ButtonData::Callback(label, callback_data) => {
                    // Check if callback_data exceeds 64 bytes or contains non-ASCII
                    let needs_storage = callback_data.len() > 64 || !callback_data.is_ascii();

                    let final_callback_data = if needs_storage {
                        // Store in storage and get reference
                        storage
                            .store_callback_data(chat_id, message_id, button_pos, callback_data)
                            .await
                    } else {
                        callback_data
                    };

                    button_row.push(InlineKeyboardButton::callback(label, final_callback_data));
                    button_pos += 1;
                }
                ButtonData::SwitchInlineQuery(label, query) => {
                    button_row.push(InlineKeyboardButton::switch_inline_query_current_chat(
                        label, query,
                    ));
                    // Don't increment button_pos for inline query buttons as they don't use storage
                }
            }
        }
        button_rows.push(button_row);
    }

    InlineKeyboardMarkup::new(button_rows)
}

/// Unpack callback data from a button press, retrieving the original data from storage if needed.
///
/// # Arguments
/// * `storage` - The callback data storage trait
/// * `callback_data` - The callback data string from the button press
///
/// # Returns
/// The original callback data string, or the input if it wasn't a storage reference
pub async fn unpack_callback_data(
    storage: &Arc<dyn CallbackDataStorageTrait>,
    callback_data: &str,
) -> String {
    // Check if it's a storage reference (starts with "cb:")
    if callback_data.starts_with("cb:") {
        // Try to retrieve from storage
        if let Some(original) = storage.get_callback_data(callback_data).await {
            return original;
        }
    }
    // Not a reference or not found in storage, return as-is
    callback_data.to_string()
}
