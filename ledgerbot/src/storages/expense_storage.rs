use std::{collections::HashMap, sync::Arc};

use chrono::NaiveDate;
// use serde::{Deserialize, Serialize}; // Commented out - serialization not supported yet
use teloxide::types::ChatId;
use tokio::sync::Mutex;

use super::{ExpensePeriod, StorageTrait};

// Type alias to simplify complex nested HashMap type
pub(crate) type ExpenseData = Arc<Mutex<HashMap<ChatId, HashMap<String, Vec<Expense>>>>>;

/// Helper function to get the current period for a chat
/// Returns the selected period from VariableStorage, or current month if not set
pub async fn get_current_period(storage: &Arc<dyn StorageTrait>, chat_id: ChatId) -> ExpensePeriod {
    let var_storage = storage.clone().as_variable_storage();
    var_storage
        .get::<ExpensePeriod>(chat_id)
        .await
        .unwrap_or_else(ExpensePeriod::current)
}

// #[derive(Debug, Clone, Serialize, Deserialize)] // Commented out - serialization not supported yet
#[derive(Debug, Clone)]
pub struct Expense {
    pub date: NaiveDate,
    pub description: String,
    pub amount: f64,
}

impl Expense {
    /// Convenient constructor for creating expense records
    pub fn new(date: NaiveDate, description: String, amount: f64) -> Self {
        Self {
            date,
            description,
            amount,
        }
    }

    /// Create expense from timestamp (for backward compatibility during migration)
    pub fn from_timestamp(timestamp: i64, description: String, amount: f64) -> Self {
        use chrono::{TimeZone, Utc};
        let datetime = Utc.timestamp_opt(timestamp, 0).unwrap();
        Self {
            date: datetime.date_naive(),
            description,
            amount,
        }
    }

    /// Get Unix timestamp for this expense (for backward compatibility)
    pub fn timestamp(&self) -> i64 {
        self.date
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp()
    }
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
    data: ExpenseData,
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
    async fn get_expenses(&self, chat_id: ChatId, period: String) -> Vec<Expense> {
        let storage_guard = self.data.lock().await;
        storage_guard
            .get(&chat_id)
            .and_then(|periods| periods.get(&period))
            .cloned()
            .unwrap_or_default()
    }

    async fn add_expenses(&self, chat_id: ChatId, period: String, expenses: Vec<Expense>) {
        let mut storage_guard = self.data.lock().await;
        let chat_data = storage_guard.entry(chat_id).or_default();
        let period_expenses = chat_data.entry(period).or_default();
        period_expenses.extend(expenses);
    }

    async fn clear_expenses(&self, chat_id: ChatId, period: String) {
        let mut storage_guard = self.data.lock().await;
        if let Some(chat_data) = storage_guard.get_mut(&chat_id) {
            chat_data.remove(&period);
        }
    }

    async fn list_periods(&self, chat_id: ChatId) -> Vec<String> {
        let storage_guard = self.data.lock().await;
        storage_guard
            .get(&chat_id)
            .map(|periods| periods.keys().cloned().collect())
            .unwrap_or_default()
    }
}

// COMMENTED OUT - Selected period concept will be used in future iteration
// #[async_trait::async_trait]
// impl ExpenseStorageTrait for ExpenseStorage {
//     async fn get_period_expenses(&self, chat_id: ChatId) -> Vec<Expense> {
//         let storage_guard = self.data.lock().await;
//         let period_guard = self.selected_period.lock().await;
//
//         let all_expenses = storage_guard.get(&chat_id).cloned().unwrap_or_default();
//
//         // If no period is selected, return all expenses
//         let Some(selected_period) = period_guard.get(&chat_id) else {
//             return all_expenses;
//         };
//
//         // Filter by selected period
//         all_expenses
//             .into_iter()
//             .filter(|e| &e.period == selected_period)
//             .collect()
//     }
//
//     async fn get_all_expenses(&self, chat_id: ChatId) -> Vec<Expense> {
//         let storage_guard = self.data.lock().await;
//         storage_guard.get(&chat_id).cloned().unwrap_or_default()
//     }
//
//     async fn add_expenses(&self, chat_id: ChatId, expenses: Vec<(String, f64, i64)>) {
//         let mut storage_guard = self.data.lock().await;
//         let period_guard = self.selected_period.lock().await;
//
//         let period = period_guard
//             .get(&chat_id)
//             .cloned()
//             .unwrap_or_else(|| "default".to_string());
//
//         let chat_expenses = storage_guard.entry(chat_id).or_default();
//         for (description, amount, timestamp) in expenses {
//             chat_expenses.push(Expense {
//                 description,
//                 amount,
//                 timestamp,
//                 period: period.clone(),
//             });
//         }
//     }
//
//     async fn add_expense(&self, chat_id: ChatId, description: &str, amount: f64, timestamp: i64) {
//         self.add_expenses(chat_id, vec![(description.to_string(), amount, timestamp)])
//             .await;
//     }
//
//     async fn clear_expenses(&self, chat_id: ChatId) {
//         let mut storage_guard = self.data.lock().await;
//         let period_guard = self.selected_period.lock().await;
//
//         // If no period selected, clear all
//         let Some(selected_period) = period_guard.get(&chat_id) else {
//             storage_guard.remove(&chat_id);
//             return;
//         };
//
//         // Remove only expenses in the selected period
//         if let Some(chat_expenses) = storage_guard.get_mut(&chat_id) {
//             chat_expenses.retain(|e| &e.period != selected_period);
//         }
//     }
//
//     async fn select_period(&self, chat_id: ChatId, period: String) {
//         let mut period_guard = self.selected_period.lock().await;
//         period_guard.insert(chat_id, period);
//     }
//
//     async fn get_selected_period(&self, chat_id: ChatId) -> Option<String> {
//         let period_guard = self.selected_period.lock().await;
//         period_guard.get(&chat_id).cloned()
//     }
// }
