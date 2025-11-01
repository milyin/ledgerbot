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
    /// Get expenses for a specific chat for the named period
    async fn get_expenses(&self, chat_id: ChatId, period: String) -> Vec<Expense>;

    /// Add expenses to a specific chat's storage for the named period
    async fn add_expenses(&self, chat_id: ChatId, period: String, expenses: Vec<Expense>);

    /// Clear all expenses for a specific chat for the named period
    async fn clear_expenses(&self, chat_id: ChatId, period: String);

    /// Get all periods available for a chat.
    async fn list_periods(&self, chat_id: ChatId) -> Vec<String>;
}

/// Per-chat storage for expenses - each chat has its own expense list
#[derive(Clone)]
pub struct ExpenseStorage {
    data: Arc<Mutex<HashMap<ChatId, HashMap<String, Vec<Expense>>>>>,
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
    async fn get_period_expenses(&self, chat_id: ChatId) -> Vec<Expense> {
        let storage_guard = self.data.lock().await;
        let period_guard = self.selected_period.lock().await;

        let all_expenses = storage_guard.get(&chat_id).cloned().unwrap_or_default();

        // If no period is selected, return all expenses
        let Some(selected_period) = period_guard.get(&chat_id) else {
            return all_expenses;
        };

        // Filter by selected period
        all_expenses
            .into_iter()
            .filter(|e| &e.period == selected_period)
            .collect()
    }

    async fn get_all_expenses(&self, chat_id: ChatId) -> Vec<Expense> {
        let storage_guard = self.data.lock().await;
        storage_guard.get(&chat_id).cloned().unwrap_or_default()
    }

    async fn add_expenses(&self, chat_id: ChatId, expenses: Vec<(String, f64, i64)>) {
        let mut storage_guard = self.data.lock().await;
        let period_guard = self.selected_period.lock().await;

        let period = period_guard
            .get(&chat_id)
            .cloned()
            .unwrap_or_else(|| "default".to_string());

        let chat_expenses = storage_guard.entry(chat_id).or_default();
        for (description, amount, timestamp) in expenses {
            chat_expenses.push(Expense {
                description,
                amount,
                timestamp,
                period: period.clone(),
            });
        }
    }

    async fn add_expense(&self, chat_id: ChatId, description: &str, amount: f64, timestamp: i64) {
        self.add_expenses(chat_id, vec![(description.to_string(), amount, timestamp)])
            .await;
    }

    async fn clear_expenses(&self, chat_id: ChatId) {
        let mut storage_guard = self.data.lock().await;
        let period_guard = self.selected_period.lock().await;

        // If no period selected, clear all
        let Some(selected_period) = period_guard.get(&chat_id) else {
            storage_guard.remove(&chat_id);
            return;
        };

        // Remove only expenses in the selected period
        if let Some(chat_expenses) = storage_guard.get_mut(&chat_id) {
            chat_expenses.retain(|e| &e.period != selected_period);
        }
    }

    async fn select_period(&self, chat_id: ChatId, period: String) {
        let mut period_guard = self.selected_period.lock().await;
        period_guard.insert(chat_id, period);
    }

    async fn get_selected_period(&self, chat_id: ChatId) -> Option<String> {
        let period_guard = self.selected_period.lock().await;
        period_guard.get(&chat_id).cloned()
    }
}
