use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use teloxide::types::ChatId;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expense {
    pub timestamp: i64,
    pub description: String,
    pub amount: f64,
    pub period: String,
}

/// Trait for expense storage operations
#[async_trait::async_trait]
pub trait ExpenseStorageTrait: Send + Sync {
    /// Get expenses for a specific chat for the named period
    async fn get_period_expenses(&self, period: String, chat_id: ChatId) -> Vec<Expense>;

    /// Add expenses to a specific chat's storage for the named period
    async fn add_period_expenses(&self, chat_id: ChatId, period: String, expenses: Vec<(String, f64, i64)>);

    /// Clear all expenses for a specific chat for the named period
    async fn clear_period_expenses(&self, chat_id: ChatId, period: String);

    /// Get all periods available for a chat. List can't be empty. On the startup
    /// the default period with name "YYYY-MM" is created and selected if no periods exist.
    async fn list_periods(&self, chat_id: ChatId) -> Vec<String>;

    /// Select the current period for a chat.
    async fn select_period(&self, chat_id: ChatId, period: String);

    /// Get the selected period for a chat. By default the period
    /// last by alphabetical order is selected on startup.
    async fn get_selected_period(&self, chat_id: ChatId) -> String;
}

/// Per-chat storage for expenses - each chat has its own expense list
#[derive(Clone)]
pub struct ExpenseStorage {
    data: Arc<Mutex<HashMap<ChatId, Vec<Expense>>>>,
    selected_period: Arc<Mutex<HashMap<ChatId, String>>>,
}

impl ExpenseStorage {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
            selected_period: Arc::new(Mutex::new(HashMap::new())),
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
