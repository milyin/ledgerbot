use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use teloxide::types::ChatId;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expense {
    pub timestamp: i64,
    pub description: String,
    pub amount: f64,
}

/// Trait for expense storage operations
#[async_trait::async_trait]
pub trait ExpenseStorageTrait: Send + Sync {
    /// Get expenses for a specific chat
    async fn get_chat_expenses(&self, chat_id: ChatId) -> Vec<Expense>;

    /// Add expenses to a specific chat's storage
    async fn add_expenses(&self, chat_id: ChatId, expenses: Vec<(String, f64, i64)>);

    /// Add a single expense
    async fn add_expense(&self, chat_id: ChatId, description: &str, amount: f64, timestamp: i64);

    /// Clear all expenses for a specific chat
    async fn clear_chat_expenses(&self, chat_id: ChatId);
}

/// Per-chat storage for expenses - each chat has its own expense list
#[derive(Clone)]
pub struct ExpenseStorage {
    data: Arc<Mutex<HashMap<ChatId, Vec<Expense>>>>,
}

impl ExpenseStorage {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

/// Implement ExpenseStorageTrait for ExpenseStorage
#[async_trait::async_trait]
impl ExpenseStorageTrait for ExpenseStorage {
    async fn get_chat_expenses(&self, chat_id: ChatId) -> Vec<Expense> {
        let storage_guard = self.data.lock().await;
        storage_guard.get(&chat_id).cloned().unwrap_or_default()
    }

    async fn add_expenses(&self, chat_id: ChatId, expenses: Vec<(String, f64, i64)>) {
        let mut storage_guard = self.data.lock().await;
        let chat_expenses = storage_guard.entry(chat_id).or_default();
        for (description, amount, timestamp) in expenses {
            chat_expenses.push(Expense {
                description,
                amount,
                timestamp,
            });
        }
    }

    async fn add_expense(&self, chat_id: ChatId, description: &str, amount: f64, timestamp: i64) {
        self.add_expenses(chat_id, vec![(description.to_string(), amount, timestamp)])
            .await;
    }

    async fn clear_chat_expenses(&self, chat_id: ChatId) {
        let mut storage_guard = self.data.lock().await;
        storage_guard.remove(&chat_id);
    }
}
