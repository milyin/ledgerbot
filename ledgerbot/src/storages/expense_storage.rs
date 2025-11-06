use std::sync::Arc;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use teloxide::types::ChatId;
use yoroolbot::storage::DataStoreTrait;

use crate::storages::Stores;

use super::ExpensePeriod;

/// Type alias for expense data (list of expenses for a period)
pub type ExpenseData = Vec<Expense>;

/// Helper function to get the current period for a chat
/// Returns the selected period from VariableStorage, or current month if not set
pub async fn get_current_period(storage: &Arc<Stores>, chat_id: ChatId) -> ExpensePeriod {
    let var_storage = storage.clone().variable_storage();
    var_storage
        .get::<ExpensePeriod>(chat_id)
        .await
        .unwrap_or_else(ExpensePeriod::current)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expense {
    pub date: NaiveDate,
    pub description: String,
    pub amount: Decimal,
}

impl Expense {
    /// Convenient constructor for creating expense records
    pub fn new(date: NaiveDate, description: String, amount: Decimal) -> Self {
        Self {
            date,
            description,
            amount,
        }
    }

    /// Create expense from timestamp (for backward compatibility during migration)
    pub fn from_timestamp(timestamp: i64, description: String, amount: Decimal) -> Self {
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
pub trait ExpenseStorageReadTrait: Send + Sync {
    /// Get all available expenses for a specific chat
    async fn get_all_expenses(&self) -> Vec<(ExpensePeriod, Expense)> {
        let periods = self.list_periods().await;
        let mut all_expenses = Vec::new();
        for period in periods {
            let expenses = self.get_expenses(period).await;
            for expense in expenses {
                all_expenses.push((period, expense));
            }
        }
        all_expenses
    }

    /// Get expenses for a specific chat for the named period
    async fn get_expenses(&self, period: ExpensePeriod) -> Vec<Expense>;

    /// Get all periods available for a chat in chronological order
    async fn list_periods(&self) -> Vec<ExpensePeriod>;
}

/// Trait for expense storage operations
#[async_trait::async_trait]
pub trait ExpenseStorageTrait: ExpenseStorageReadTrait + Send + Sync {
    /// Add expenses to a specific chat's storage for the named period
    async fn add_expenses(&self, period: ExpensePeriod, expenses: Vec<Expense>);

    /// Clear all expenses for a specific chat for the named period
    async fn clear_expenses(&self, period: ExpensePeriod);
}

/// Generic expense storage that works with any DataStore implementation
/// Each period is stored as a separate key (period string) with its expenses as the value
#[derive(Clone)]
pub struct ExpenseStorage
{
    store: Arc<dyn DataStoreTrait<ExpenseData>>,
    chat_id: ChatId,
}

impl ExpenseStorage
{
    /// Create a new ExpenseStorage with the given DataStore and chat ID
    pub fn new(store: Arc<dyn DataStoreTrait<ExpenseData>>, chat_id: ChatId) -> Self {
        Self { store, chat_id }
    }
}

/// Implement ExpenseStorageTrait for ExpenseStorage
#[async_trait::async_trait]
impl ExpenseStorageReadTrait for ExpenseStorage {
    async fn get_expenses(&self, period: ExpensePeriod) -> Vec<Expense> {
        let period = period.to_string();
        self.store.get(self.chat_id, &period).await.unwrap_or_default()
    }

    async fn list_periods(&self) -> Vec<ExpensePeriod> {
        let mut periods = self
            .store
            .keys(self.chat_id)
            .await
            .into_iter()
            .filter_map(|key| ExpensePeriod::from_string(&key).ok())
            .collect::<Vec<_>>();
        periods.sort();
        periods
    }
}

/// Implement ExpenseStorageTrait for ExpenseStorage
#[async_trait::async_trait]
impl ExpenseStorageTrait for ExpenseStorage {
    async fn add_expenses(&self, period: ExpensePeriod, expenses: Vec<Expense>) {
        let period = period.to_string();
        let mut period_expenses = self.store.get(self.chat_id, &period).await.unwrap_or_default();
        period_expenses.extend(expenses);
        self.store.set(self.chat_id, &period, period_expenses).await;
    }

    async fn clear_expenses(&self, period: ExpensePeriod) {
        let period = period.to_string();
        self.store.remove(self.chat_id, &period).await;
    }
}
