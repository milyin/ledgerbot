use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use teloxide::types::ChatId;
use tokio::sync::Mutex;

/// Shared data structure for all variable storage
/// Outer map: ChatId -> Inner map of variables
/// Inner map: TypeId -> Arc<dyn Any> (the actual value)
pub type VariableData = Arc<Mutex<HashMap<ChatId, HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>>;

/// Storage for per-chat typed variables using TypeId as key
/// This allows storing different types of data per chat without type erasure
#[derive(Clone)]
pub struct VariableStorage {
    data: VariableData,
    chat_id: ChatId,
}

impl VariableStorage {
    /// Create a new VariableStorage with the given shared data and chat ID
    pub fn new(data: VariableData, chat_id: ChatId) -> Self {
        Self { data, chat_id }
    }

    /// Get a variable of type T
    /// Returns None if the variable doesn't exist or has a different type
    pub async fn get<T: Any + Send + Sync + Clone>(&self) -> Option<T> {
        let data_guard = self.data.lock().await;
        let chat_vars = data_guard.get(&self.chat_id)?;
        let type_id = TypeId::of::<T>();
        let any_value = chat_vars.get(&type_id)?;

        // Downcast from Arc<dyn Any> to Arc<T>, then clone the inner value
        any_value.downcast_ref::<T>().cloned()
    }

    /// Set a variable of type T
    /// Overwrites any existing value of the same type
    pub async fn set<T: Any + Send + Sync>(&self, value: T) {
        let mut data_guard = self.data.lock().await;
        let chat_vars = data_guard.entry(self.chat_id).or_insert_with(HashMap::new);
        let type_id = TypeId::of::<T>();
        chat_vars.insert(type_id, Arc::new(value));
    }

    /// Remove a variable of type T
    /// Returns true if the variable existed and was removed
    pub async fn remove<T: Any + Send + Sync>(&self) -> bool {
        let mut data_guard = self.data.lock().await;
        if let Some(chat_vars) = data_guard.get_mut(&self.chat_id) {
            let type_id = TypeId::of::<T>();
            chat_vars.remove(&type_id).is_some()
        } else {
            false
        }
    }

    /// Clear all variables for this chat
    pub async fn clear(&self) {
        let mut data_guard = self.data.lock().await;
        data_guard.remove(&self.chat_id);
    }
}

// Note: We don't create a trait for VariableStorage because traits with generic methods
// cannot be made into trait objects (not dyn-compatible). Instead, we use the concrete
// VariableStorage type directly throughout the codebase.

#[cfg(test)]
mod tests {
    use teloxide::types::ChatId;

    use super::*;

    #[tokio::test]
    async fn test_set_and_get() {
        let chat_id = ChatId(123);
        let data = Arc::new(Mutex::new(HashMap::new()));
        let storage = VariableStorage::new(data, chat_id);

        // Set a String value
        storage.set("test_value".to_string()).await;

        // Get the String value back
        let result: Option<String> = storage.get().await;
        assert_eq!(result, Some("test_value".to_string()));
    }

    #[tokio::test]
    async fn test_multiple_types() {
        let chat_id = ChatId(123);
        let data = Arc::new(Mutex::new(HashMap::new()));
        let storage = VariableStorage::new(data, chat_id);

        // Set different types
        storage.set("string_value".to_string()).await;
        storage.set(42i32).await;
        storage.set(true).await;

        // Get them back
        let string_val: Option<String> = storage.get().await;
        let int_val: Option<i32> = storage.get().await;
        let bool_val: Option<bool> = storage.get().await;

        assert_eq!(string_val, Some("string_value".to_string()));
        assert_eq!(int_val, Some(42));
        assert_eq!(bool_val, Some(true));
    }

    #[tokio::test]
    async fn test_overwrite_value() {
        let chat_id = ChatId(123);
        let data = Arc::new(Mutex::new(HashMap::new()));
        let storage = VariableStorage::new(data, chat_id);

        // Set initial value
        storage.set("first".to_string()).await;
        let first: Option<String> = storage.get().await;
        assert_eq!(first, Some("first".to_string()));

        // Overwrite with new value
        storage.set("second".to_string()).await;
        let second: Option<String> = storage.get().await;
        assert_eq!(second, Some("second".to_string()));
    }

    #[tokio::test]
    async fn test_get_nonexistent() {
        let chat_id = ChatId(123);
        let data = Arc::new(Mutex::new(HashMap::new()));
        let storage = VariableStorage::new(data, chat_id);

        // Try to get a value that was never set
        let result: Option<String> = storage.get().await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_remove() {
        let chat_id = ChatId(123);
        let data = Arc::new(Mutex::new(HashMap::new()));
        let storage = VariableStorage::new(data, chat_id);

        // Set and verify
        storage.set("test".to_string()).await;
        let before: Option<String> = storage.get().await;
        assert_eq!(before, Some("test".to_string()));

        // Remove and verify
        let removed = storage.remove::<String>().await;
        assert!(removed);
        let after: Option<String> = storage.get().await;
        assert_eq!(after, None);

        // Try to remove again
        let removed_again = storage.remove::<String>().await;
        assert!(!removed_again);
    }

    #[tokio::test]
    async fn test_clear_chat() {
        let chat_id = ChatId(123);
        let data = Arc::new(Mutex::new(HashMap::new()));
        let storage = VariableStorage::new(data, chat_id);

        // Set multiple types
        storage.set("string".to_string()).await;
        storage.set(42i32).await;

        // Verify they exist
        assert!(storage.get::<String>().await.is_some());
        assert!(storage.get::<i32>().await.is_some());

        // Clear all
        storage.clear().await;

        // Verify they're gone
        assert!(storage.get::<String>().await.is_none());
        assert!(storage.get::<i32>().await.is_none());
    }

    #[tokio::test]
    async fn test_multiple_chats() {
        let data = Arc::new(Mutex::new(HashMap::new()));
        let chat1 = ChatId(123);
        let chat2 = ChatId(456);
        let storage1 = VariableStorage::new(data.clone(), chat1);
        let storage2 = VariableStorage::new(data, chat2);

        // Set different values for different chats
        storage1.set("chat1_value".to_string()).await;
        storage2.set("chat2_value".to_string()).await;

        // Verify isolation
        let val1: Option<String> = storage1.get().await;
        let val2: Option<String> = storage2.get().await;

        assert_eq!(val1, Some("chat1_value".to_string()));
        assert_eq!(val2, Some("chat2_value".to_string()));
    }
}
