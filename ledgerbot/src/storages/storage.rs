use std::sync::Arc;

use teloxide::types::ChatId;
use yoroolbot::storage::{CallbackDataStorage, CallbackDataStorageTrait, DataStoreTrait, InMemStore};

use crate::storages::{
    BatchStorage, BatchStorageTrait, CategoryData, CategoryStorage, CategoryStorageTrait,
    ExpenseData, ExpenseStorage, ExpenseStorageTrait, ShareData, ShareStorage, ShareStorageTrait,
    VariableStorage,
};

/// Main storage structure that holds all bot data
/// This is the primary storage container for the application
#[derive(Clone)]
pub struct Stores {
    expenses_data_store: Arc<dyn DataStoreTrait<ExpenseData>>,
    categories_data_store: Arc<dyn DataStoreTrait<CategoryData>>,
    shares_data_store: Arc<dyn DataStoreTrait<ShareData>>,
    batch: Arc<dyn BatchStorageTrait>,
    callback_data: Arc<dyn CallbackDataStorageTrait>,
    variables: Arc<VariableStorage>,
}

impl Stores {
    /// Create a new storage with all storage types initialized (in-memory)
    pub fn new() -> Self {
        let expenses_data_store = Arc::new(InMemStore::<ExpenseData>::new());
        let categories_data_store = Arc::new(InMemStore::<CategoryData>::new());
        let shares_data_store = Arc::new(InMemStore::<ShareData>::new());
        Self {
            expenses_data_store,
            categories_data_store,
            shares_data_store,
            batch: Arc::new(BatchStorage::new()),
            callback_data: Arc::new(CallbackDataStorage::new()),
            variables: Arc::new(VariableStorage::new()),
        }
    }

    pub fn storage(&self, chat_id: ChatId) -> Arc<Storage> {
        Arc::new(Storage {
            chat_id,
            stores: self.clone(),
        })
    }

    /// Builder-like method to configure expense storage
    /// Replaces the expense storage with the provided implementation
    pub fn expenses_store(mut self, store: impl DataStoreTrait<ExpenseData> + 'static) -> Self {
        self.expenses_data_store = Arc::new(store);
        self
    }

    /// Builder-like method to configure category storage
    /// Replaces the category storage with the provided implementation
    pub fn categories_store(mut self, store: impl DataStoreTrait<CategoryData> + 'static) -> Self {
        self.categories_data_store = Arc::new(store);
        self
    }

    /// Builder-like method to configure share storage
    /// Replaces the share storage with the provided implementation
    pub fn shares_store(mut self, store: impl DataStoreTrait<ShareData> + 'static) -> Self {
        self.shares_data_store = Arc::new(store);
        self
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

impl Default for Stores {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct Storage {
    pub chat_id: ChatId,
    pub stores: Stores,
}

impl Storage {
    /// Get expense storage for this chat
    pub fn expenses(&self) -> Arc<dyn ExpenseStorageTrait> {
        Arc::new(ExpenseStorage::new(
            self.stores.expenses_data_store.clone(),
            self.chat_id,
        ))
    }

    /// Get category storage for this chat
    pub fn categories(&self) -> Arc<dyn CategoryStorageTrait> {
        Arc::new(CategoryStorage::new(
            self.stores.categories_data_store.clone(),
            self.chat_id,
        ))
    }

    /// Get share storage for this chat
    pub fn shares(&self) -> Arc<dyn ShareStorageTrait> {
        Arc::new(ShareStorage::new(
            self.stores.shares_data_store.clone(),
            self.chat_id,
        ))
    }
}