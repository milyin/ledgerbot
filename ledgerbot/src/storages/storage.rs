use std::sync::Arc;

use yoroolbot::storage::{CallbackDataStorage, CallbackDataStorageTrait};

use super::category_storage::CategoryStorage;
use crate::storages::{
    BatchStorage, BatchStorageTrait, CategoryStorageTrait, ExpenseStorage, ExpenseStorageTrait, PeriodsStorage, PeriodsStorageTrait
};

/// Combined storage trait that provides all storage operations
/// This trait allows converting to specific trait objects for functions that only need subset of functionality
pub trait StorageTrait: Send + Sync {
    /// Convert to ExpenseStorageTrait trait object
    fn as_expense_storage(self: Arc<Self>) -> Arc<dyn ExpenseStorageTrait>;

    /// Convert to CategoryStorageTrait trait object
    fn as_category_storage(self: Arc<Self>) -> Arc<dyn CategoryStorageTrait>;

    /// Convert to BatchStorageTrait trait object
    fn as_batch_storage(self: Arc<Self>) -> Arc<dyn BatchStorageTrait>;

    /// Convert to CallbackDataStorageTrait trait object
    fn as_callback_data_storage(self: Arc<Self>) -> Arc<dyn CallbackDataStorageTrait>;

    /// Convert to PeriodsStorageTrait trait object
    fn as_periods_storage(self: Arc<Self>) -> Arc<dyn PeriodsStorageTrait>;
}

/// Main storage structure that holds all bot data
/// This is the primary storage container for the application
#[derive(Clone)]
pub struct Storage {
    expenses: Arc<dyn ExpenseStorageTrait>,
    categories: Arc<dyn CategoryStorageTrait>,
    batch: Arc<dyn BatchStorageTrait>,
    callback_data: Arc<dyn CallbackDataStorageTrait>,
    periods: Arc<dyn PeriodsStorageTrait>,
}

impl Storage {
    /// Create a new storage with all storage types initialized (in-memory)
    pub fn new() -> Self {
        Self {
            expenses: Arc::new(ExpenseStorage::new()),
            categories: Arc::new(CategoryStorage::new()),
            batch: Arc::new(BatchStorage::new()),
            callback_data: Arc::new(CallbackDataStorage::new()),
            periods: Arc::new(PeriodsStorage::new()),
        }
    }

    /// Builder-like method to configure category storage
    /// Replaces the category storage with the provided implementation
    pub fn categories_storage(mut self, storage: impl CategoryStorageTrait + 'static) -> Self {
        self.categories = Arc::new(storage);
        self
    }

    /// Builder-like method to configure periods storage
    /// Replaces the periods storage with the provided implementation
    pub fn periods_storage(mut self, storage: impl PeriodsStorageTrait + 'static) -> Self {
        self.periods = Arc::new(storage);
        self
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}

/// Implement StorageTrait for Storage to enable conversion to specific trait objects
impl StorageTrait for Storage {
    fn as_expense_storage(self: Arc<Self>) -> Arc<dyn ExpenseStorageTrait> {
        self.expenses.clone()
    }

    fn as_category_storage(self: Arc<Self>) -> Arc<dyn CategoryStorageTrait> {
        self.categories.clone()
    }

    fn as_batch_storage(self: Arc<Self>) -> Arc<dyn BatchStorageTrait> {
        self.batch.clone()
    }

    fn as_callback_data_storage(self: Arc<Self>) -> Arc<dyn CallbackDataStorageTrait> {
        self.callback_data.clone()
    }

    fn as_periods_storage(self: Arc<Self>) -> Arc<dyn PeriodsStorageTrait> {
        self.periods.clone()
    }
}
