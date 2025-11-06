use std::sync::Arc;

use yoroolbot::storage::{CallbackDataStorage, CallbackDataStorageTrait, InMemStore};

use crate::storages::{
    BatchStorage, BatchStorageTrait, CategoryData, CategoryStorage, CategoryStorageTrait,
    ExpenseData, ExpenseStorage, ExpenseStorageTrait, ShareData, ShareStorage, ShareStorageTrait,
    VariableStorage,
};

/// Main storage structure that holds all bot data
/// This is the primary storage container for the application
#[derive(Clone)]
pub struct Storage {
    expenses: Arc<dyn ExpenseStorageTrait>,
    categories: Arc<dyn CategoryStorageTrait>,
    shares: Arc<dyn ShareStorageTrait>,
    batch: Arc<dyn BatchStorageTrait>,
    callback_data: Arc<dyn CallbackDataStorageTrait>,
    variables: Arc<VariableStorage>,
}

impl Storage {
    /// Create a new storage with all storage types initialized (in-memory)
    pub fn new() -> Self {
        Self {
            expenses: Arc::new(ExpenseStorage::new(InMemStore::<ExpenseData>::new())),
            categories: Arc::new(CategoryStorage::new(InMemStore::<CategoryData>::new())),
            shares: Arc::new(ShareStorage::new(InMemStore::<ShareData>::new())),
            batch: Arc::new(BatchStorage::new()),
            callback_data: Arc::new(CallbackDataStorage::new()),
            variables: Arc::new(VariableStorage::new()),
        }
    }

    /// Builder-like method to configure expense storage
    /// Replaces the expense storage with the provided implementation
    pub fn expenses_storage(mut self, storage: impl ExpenseStorageTrait + 'static) -> Self {
        self.expenses = Arc::new(storage);
        self
    }

    /// Builder-like method to configure category storage
    /// Replaces the category storage with the provided implementation
    pub fn categories_storage(mut self, storage: impl CategoryStorageTrait + 'static) -> Self {
        self.categories = Arc::new(storage);
        self
    }

    /// Builder-like method to configure share storage
    /// Replaces the share storage with the provided implementation
    pub fn shares_storage(mut self, storage: impl ShareStorageTrait + 'static) -> Self {
        self.shares = Arc::new(storage);
        self
    }

    /// Get expense storage
    pub fn expense_storage(self: &Arc<Self>) -> Arc<dyn ExpenseStorageTrait> {
        self.expenses.clone()
    }

    /// Get category storage
    pub fn category_storage(self: &Arc<Self>) -> Arc<dyn CategoryStorageTrait> {
        self.categories.clone()
    }

    /// Get share storage
    pub fn share_storage(self: &Arc<Self>) -> Arc<dyn ShareStorageTrait> {
        self.shares.clone()
    }

    /// Get batch storage
    pub fn batch_storage(self: &Arc<Self>) -> Arc<dyn BatchStorageTrait> {
        self.batch.clone()
    }

    /// Get callback data storage
    pub fn callback_data_storage(self: &Arc<Self>) -> Arc<dyn CallbackDataStorageTrait> {
        self.callback_data.clone()
    }

    /// Get variable storage
    pub fn variable_storage(self: &Arc<Self>) -> Arc<VariableStorage> {
        self.variables.clone()
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}
