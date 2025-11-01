use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use teloxide::types::ChatId;
use tokio::sync::Mutex;

/// Storage for per-chat typed variables using TypeId as key
/// This allows storing different types of data per chat without type erasure
#[derive(Clone)]
pub struct VariableStorage {
    // Outer map: ChatId -> Inner map of variables
    // Inner map: TypeId -> Arc<dyn Any> (the actual value)
    data: Arc<Mutex<HashMap<ChatId, HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>>,
}

impl VariableStorage {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get a variable of type T for a specific chat
    /// Returns None if the variable doesn't exist or has a different type
    pub async fn get<T: Any + Send + Sync + Clone>(&self, chat_id: ChatId) -> Option<T> {
        let data_guard = self.data.lock().await;
        let chat_vars = data_guard.get(&chat_id)?;
        let type_id = TypeId::of::<T>();
        let any_value = chat_vars.get(&type_id)?;

        // Downcast from Arc<dyn Any> to Arc<T>, then clone the inner value
        any_value.downcast_ref::<T>().cloned()
    }

    /// Set a variable of type T for a specific chat
    /// Overwrites any existing value of the same type
    pub async fn set<T: Any + Send + Sync>(&self, chat_id: ChatId, value: T) {
        let mut data_guard = self.data.lock().await;
        let chat_vars = data_guard.entry(chat_id).or_insert_with(HashMap::new);
        let type_id = TypeId::of::<T>();
        chat_vars.insert(type_id, Arc::new(value));
    }

    /// Remove a variable of type T for a specific chat
    /// Returns true if the variable existed and was removed
    pub async fn remove<T: Any + Send + Sync>(&self, chat_id: ChatId) -> bool {
        let mut data_guard = self.data.lock().await;
        if let Some(chat_vars) = data_guard.get_mut(&chat_id) {
            let type_id = TypeId::of::<T>();
            chat_vars.remove(&type_id).is_some()
        } else {
            false
        }
    }

    /// Clear all variables for a specific chat
    pub async fn clear_chat(&self, chat_id: ChatId) {
        let mut data_guard = self.data.lock().await;
        data_guard.remove(&chat_id);
    }
}

// Note: We don't create a trait for VariableStorage because traits with generic methods
// cannot be made into trait objects (not dyn-compatible). Instead, we use the concrete
// VariableStorage type directly throughout the codebase.

#[cfg(test)]
mod tests {
    use super::*;
    use teloxide::types::ChatId;

    #[tokio::test]
    async fn test_set_and_get() {
        let storage = VariableStorage::new();
        let chat_id = ChatId(123);

        // Set a String value
        storage.set(chat_id, "test_value".to_string()).await;

        // Get the String value back
        let result: Option<String> = storage.get(chat_id).await;
        assert_eq!(result, Some("test_value".to_string()));
    }

    #[tokio::test]
    async fn test_multiple_types() {
        let storage = VariableStorage::new();
        let chat_id = ChatId(123);

        // Set different types
        storage.set(chat_id, "string_value".to_string()).await;
        storage.set(chat_id, 42i32).await;
        storage.set(chat_id, true).await;

        // Get them back
        let string_val: Option<String> = storage.get(chat_id).await;
        let int_val: Option<i32> = storage.get(chat_id).await;
        let bool_val: Option<bool> = storage.get(chat_id).await;

        assert_eq!(string_val, Some("string_value".to_string()));
        assert_eq!(int_val, Some(42));
        assert_eq!(bool_val, Some(true));
    }

    #[tokio::test]
    async fn test_overwrite_value() {
        let storage = VariableStorage::new();
        let chat_id = ChatId(123);

        // Set initial value
        storage.set(chat_id, "first".to_string()).await;
        let first: Option<String> = storage.get(chat_id).await;
        assert_eq!(first, Some("first".to_string()));

        // Overwrite with new value
        storage.set(chat_id, "second".to_string()).await;
        let second: Option<String> = storage.get(chat_id).await;
        assert_eq!(second, Some("second".to_string()));
    }

    #[tokio::test]
    async fn test_get_nonexistent() {
        let storage = VariableStorage::new();
        let chat_id = ChatId(123);

        // Try to get a value that was never set
        let result: Option<String> = storage.get(chat_id).await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_remove() {
        let storage = VariableStorage::new();
        let chat_id = ChatId(123);

        // Set and verify
        storage.set(chat_id, "test".to_string()).await;
        let before: Option<String> = storage.get(chat_id).await;
        assert_eq!(before, Some("test".to_string()));

        // Remove and verify
        let removed = storage.remove::<String>(chat_id).await;
        assert!(removed);
        let after: Option<String> = storage.get(chat_id).await;
        assert_eq!(after, None);

        // Try to remove again
        let removed_again = storage.remove::<String>(chat_id).await;
        assert!(!removed_again);
    }

    #[tokio::test]
    async fn test_clear_chat() {
        let storage = VariableStorage::new();
        let chat_id = ChatId(123);

        // Set multiple types
        storage.set(chat_id, "string".to_string()).await;
        storage.set(chat_id, 42i32).await;

        // Verify they exist
        assert!(storage.get::<String>(chat_id).await.is_some());
        assert!(storage.get::<i32>(chat_id).await.is_some());

        // Clear all
        storage.clear_chat(chat_id).await;

        // Verify they're gone
        assert!(storage.get::<String>(chat_id).await.is_none());
        assert!(storage.get::<i32>(chat_id).await.is_none());
    }

    #[tokio::test]
    async fn test_multiple_chats() {
        let storage = VariableStorage::new();
        let chat1 = ChatId(123);
        let chat2 = ChatId(456);

        // Set different values for different chats
        storage.set(chat1, "chat1_value".to_string()).await;
        storage.set(chat2, "chat2_value".to_string()).await;

        // Verify isolation
        let val1: Option<String> = storage.get(chat1).await;
        let val2: Option<String> = storage.get(chat2).await;

        assert_eq!(val1, Some("chat1_value".to_string()));
        assert_eq!(val2, Some("chat2_value".to_string()));
    }
}
