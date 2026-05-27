use std::sync::Arc;

use serde::{Deserialize, Serialize};
use telluride::{
    command::{CallbackEncode, CallbackKey, UnpackError},
    data_store::DataStoreTrait as TellurideDataStoreTrait,
};
use teloxide::types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup};

use crate::storage::DataStoreTrait;

/// Callback payload stored for inline keyboard actions.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CallbackData(String);

impl CallbackData {
    pub fn new(data: impl Into<String>) -> Self {
        Self(data.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<String> for CallbackData {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for CallbackData {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl CallbackEncode for CallbackData {
    fn encode_callback(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }

    fn decode_callback(bytes: &[u8]) -> Result<Self, UnpackError> {
        String::from_utf8(bytes.to_vec())
            .map(Self)
            .map_err(|e| UnpackError::DeserializeError(e.to_string()))
    }
}

/// Represents different types of inline keyboard buttons.
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

/// Trait for callback data storage read operations.
#[async_trait::async_trait]
pub trait CallbackDataStorageReadTrait: Send + Sync {
    /// Retrieve original callback data from a packed reference string.
    async fn get_callback_data(&self, reference: &str) -> Option<CallbackData>;
}

/// Trait for callback data storage operations.
#[async_trait::async_trait]
pub trait CallbackDataStorageTrait: CallbackDataStorageReadTrait + Send + Sync {
    /// Store callback data and return a packed callback key.
    async fn store_callback_data(
        &self,
        message_id: i32,
        button_pos: usize,
        data: CallbackData,
    ) -> String;

    /// Clear callbacks associated with a message.
    async fn clear_message_callbacks(&self, message_id: i32);
}

#[derive(Clone)]
struct TellurideCallbackStore {
    store: Arc<dyn DataStoreTrait<CallbackData>>,
    chat_id: ChatId,
}

#[async_trait::async_trait]
impl TellurideDataStoreTrait<CallbackKey, CallbackData> for TellurideCallbackStore {
    async fn get(&self, key: &CallbackKey) -> Option<CallbackData> {
        self.store.get(self.chat_id, key.as_str()).await
    }

    async fn set(&self, key: &CallbackKey, value: CallbackData) {
        self.store.set(self.chat_id, key.as_str(), value).await;
    }

    async fn remove(&self, key: &CallbackKey) -> bool {
        self.store.remove(self.chat_id, key.as_str()).await
    }

    async fn keys(&self) -> Vec<CallbackKey> {
        self.store
            .keys(self.chat_id)
            .await
            .into_iter()
            .filter_map(|raw| raw.parse().ok())
            .collect()
    }
}

/// Callback storage implementation backed by the legacy chat-scoped store and
/// telluride's `CallbackKey` packing logic.
#[derive(Clone)]
pub struct CallbackDataStorage {
    store: Arc<dyn DataStoreTrait<CallbackData>>,
    chat_id: ChatId,
}

impl CallbackDataStorage {
    pub fn new(store: Arc<dyn DataStoreTrait<CallbackData>>, chat_id: ChatId) -> Self {
        Self { store, chat_id }
    }

    fn telluride_store(&self) -> TellurideCallbackStore {
        TellurideCallbackStore {
            store: Arc::clone(&self.store),
            chat_id: self.chat_id,
        }
    }
}

#[async_trait::async_trait]
impl CallbackDataStorageReadTrait for CallbackDataStorage {
    async fn get_callback_data(&self, reference: &str) -> Option<CallbackData> {
        let store = self.telluride_store();
        CallbackKey::unpack::<CallbackData, _>(reference, &store)
            .await
            .ok()
    }
}

#[async_trait::async_trait]
impl CallbackDataStorageTrait for CallbackDataStorage {
    async fn store_callback_data(
        &self,
        _message_id: i32,
        _button_pos: usize,
        data: CallbackData,
    ) -> String {
        let store = self.telluride_store();
        CallbackKey::pack(data, &store).await.to_string()
    }

    async fn clear_message_callbacks(&self, _message_id: i32) {
        // Telluride callback keys are value-addressed, so message-local eager
        // cleanup is not required here.
    }
}

/// Pack callback data into an `InlineKeyboardMarkup` using telluride callback keys.
pub async fn pack_callback_data<R, B>(
    storage: &Arc<dyn CallbackDataStorageTrait>,
    message_id: i32,
    rows: impl IntoIterator<Item = R>,
) -> InlineKeyboardMarkup
where
    R: IntoIterator<Item = B>,
    B: Into<ButtonData>,
{
    storage.clear_message_callbacks(message_id).await;

    let mut button_rows = Vec::new();
    let mut button_pos = 0;

    for row in rows {
        let mut button_row = Vec::new();
        for item in row {
            let button_data: ButtonData = item.into();

            match button_data {
                ButtonData::Callback(label, callback_data) => {
                    let final_callback_data = storage
                        .store_callback_data(
                            message_id,
                            button_pos,
                            CallbackData::from(callback_data),
                        )
                        .await;
                    button_row.push(InlineKeyboardButton::callback(label, final_callback_data));
                    button_pos += 1;
                }
                ButtonData::SwitchInlineQuery(label, query) => {
                    button_row.push(InlineKeyboardButton::switch_inline_query_current_chat(
                        label, query,
                    ));
                }
            }
        }
        button_rows.push(button_row);
    }

    InlineKeyboardMarkup::new(button_rows)
}

/// Unpack callback data from a button press.
pub async fn unpack_callback_data(
    storage: &Arc<dyn CallbackDataStorageTrait>,
    callback_data: &str,
) -> String {
    if let Some(original) = storage.get_callback_data(callback_data).await {
        return original.into_inner();
    }

    callback_data.to_string()
}
