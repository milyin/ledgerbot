use std::collections::HashMap;
use std::sync::Arc;
use teloxide::types::ChatId;
use tokio::sync::Mutex;

/// Per-chat storage for expenses - each chat has its own expense HashMap
pub type ExpenseStorage = Arc<Mutex<HashMap<ChatId, HashMap<String, f64>>>>;

/// Get expenses for a specific chat
pub async fn get_chat_expenses(storage: &ExpenseStorage, chat_id: ChatId) -> HashMap<String, f64> {
    let storage_guard = storage.lock().await;
    storage_guard.get(&chat_id).cloned().unwrap_or_default()
}

/// Add expenses to a specific chat's storage
pub async fn add_expenses(
    storage: &ExpenseStorage,
    chat_id: ChatId,
    expenses: Vec<(String, f64)>,
) {
    let mut storage_guard = storage.lock().await;
    let chat_expenses = storage_guard.entry(chat_id).or_default();
    for (description, amount) in expenses {
        chat_expenses.insert(description, amount);
    }
}

/// Clear all expenses for a specific chat
pub async fn clear_chat_expenses(storage: &ExpenseStorage, chat_id: ChatId) {
    let mut storage_guard = storage.lock().await;
    storage_guard.remove(&chat_id);
}

/// Create a new expense storage
pub fn create_storage() -> ExpenseStorage {
    Arc::new(Mutex::new(HashMap::new()))
}
