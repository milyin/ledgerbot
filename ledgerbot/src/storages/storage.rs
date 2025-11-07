use std::sync::Arc;

use teloxide::types::ChatId;
use yoroolbot::storage::{CallbackData, CallbackDataStorage, CallbackDataStorageTrait, DataStoreTrait, InMemStore};

use crate::storages::{
    BatchData, BatchStorage, BatchStorageTrait, CategoryData, CategoryStorage, CategoryStorageTrait, ExpenseData, ExpenseStorage, ExpenseStorageTrait, ShareData, ShareStorage, ShareStorageTrait, VariableData, VariableStorage, category_storage::CategoryStorageReadTrait, expense_storage::ExpenseStorageReadTrait, share_storage::ShareStorageReadTrait
};

/// Main storage structure that holds all bot data
/// This is the primary storage container for the application
#[derive(Clone)]
pub struct Stores {
    expenses_data_store: Arc<dyn DataStoreTrait<ExpenseData>>,
    categories_data_store: Arc<dyn DataStoreTrait<CategoryData>>,
    shares_data_store: Arc<dyn DataStoreTrait<ShareData>>,
    batch_data_store: Arc<dyn DataStoreTrait<BatchData>>,
    callback_data_store: Arc<dyn DataStoreTrait<CallbackData>>,
    variables_data: VariableData,
}

impl Stores {
    /// Create a new storage with all storage types initialized (in-memory)
    pub fn new() -> Self {
        let expenses_data_store = Arc::new(InMemStore::<ExpenseData>::new());
        let categories_data_store = Arc::new(InMemStore::<CategoryData>::new());
        let shares_data_store = Arc::new(InMemStore::<ShareData>::new());
        let batch_data_store = Arc::new(InMemStore::<BatchData>::new());
        let callback_data_store = Arc::new(InMemStore::<CallbackData>::new());
        let variables_data = VariableData::default();
        Self {
            expenses_data_store,
            categories_data_store,
            shares_data_store,
            batch_data_store,
            callback_data_store,
            variables_data,
        }
    }

    pub fn storage(&self, chat_id: ChatId) -> Arc<Storage> {
        Arc::new(Storage {
            chat_id,
            stores: self.clone(),
        })
    }

    pub fn storage_readonly(&self, own_chat_id: ChatId, external_chat_id: ChatId) -> Arc<StorageReadonly> {
        Arc::new(StorageReadonly {
            own_chat_id,
            external_chat_id,
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

    /// Builder-like method to configure batch storage
    /// Replaces the batch storage with the provided implementation
    pub fn batch_store(mut self, store: impl DataStoreTrait<BatchData> + 'static) -> Self {
        self.batch_data_store = Arc::new(store);
        self
    }

    /// Builder-like method to configure callback data storage
    /// Replaces the callback data storage with the provided implementation
    pub fn callback_data_store(mut self, store: impl DataStoreTrait<CallbackData> + 'static) -> Self {
        self.callback_data_store = Arc::new(store);
        self
    }
}

impl Default for Stores {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct Storage {
    chat_id: ChatId,
    stores: Stores,
}

impl Storage {
    /// Get expense storage for this chat
    pub fn expenses(&self) -> Arc<dyn ExpenseStorageTrait> {
        Arc::new(ExpenseStorage::new(
            self.stores.expenses_data_store.clone(),
            self.chat_id,
        ))
    }

    /// Get readonly expense storage for this chat
    pub fn expenses_readonly(&self) -> Arc<dyn ExpenseStorageReadTrait> {
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

    /// Get readonly category storage for this chat
    pub fn categories_readonly(&self) -> Arc<dyn CategoryStorageReadTrait> {
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

    /// Get readonly share storage for this chat
    pub fn shares_readonly(&self) -> Arc<dyn ShareStorageReadTrait> {
        Arc::new(ShareStorage::new(
            self.stores.shares_data_store.clone(),
            self.chat_id,
        ))
    }

    /// Get batch storage for this chat
    pub fn batch(&self) -> Arc<dyn BatchStorageTrait> {
        Arc::new(BatchStorage::new(
            self.stores.batch_data_store.clone(),
            self.chat_id,
        ))
    }

    /// Get callback data storage for this chat
    pub fn callback_data(&self) -> Arc<dyn CallbackDataStorageTrait> {
        Arc::new(CallbackDataStorage::new(
            self.stores.callback_data_store.clone(),
            self.chat_id,
        ))
    }

    /// Get variable storage for this chat
    pub fn variables(&self) -> VariableStorage {
        VariableStorage::new(self.stores.variables_data.clone(), self.chat_id)
    }
}

#[derive(Clone)]
pub struct StorageReadonly {
    own_chat_id: ChatId,
    external_chat_id: ChatId,
    stores: Stores,
}

impl StorageReadonly {
    /// Get expense storage for this chat
    pub fn expenses_readonly(&self) -> Arc<dyn ExpenseStorageReadTrait> {
        Arc::new(ExpenseStorage::new(
            self.stores.expenses_data_store.clone(),
            self.external_chat_id,
        ))
    }

    /// Get category storage for this chat
    pub fn categories_readonly(&self) -> Arc<dyn CategoryStorageReadTrait> {
        Arc::new(CategoryStorage::new(
            self.stores.categories_data_store.clone(),
            self.external_chat_id,
        ))
    }

    /// Get share storage for this chat
    pub fn shares_readonly(&self) -> Arc<dyn ShareStorageReadTrait> {
        Arc::new(ShareStorage::new(
            self.stores.shares_data_store.clone(),
            self.external_chat_id,
        ))
    }

    /// Get variable storage for this chat
    pub fn variables(&self) -> VariableStorage {
        VariableStorage::new(self.stores.variables_data.clone(), self.own_chat_id)
    }

    /// Get the external chat ID if this storage is for an external chat
    pub fn external_chat_id(&self) -> Option<ChatId> {
        if self.own_chat_id != self.external_chat_id {
            Some(self.external_chat_id)
        } else {
            None
        }
    }
}

