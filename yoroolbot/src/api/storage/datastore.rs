use serde::{Deserialize, Serialize};
use std::{collections::HashMap, marker::PhantomData, path::PathBuf, sync::Arc};
use tokio::{fs, sync::Mutex};

use crate::api::storage::utils::{decode_filename_to_key, encode_key_to_filename};

/// Trait for key-value data storage with serializable values
/// The key is always a string, the value type V must be serializable
#[async_trait::async_trait]
pub trait DataStore<V>: Send + Sync + Clone
where
    V: Serialize + for<'de> Deserialize<'de> + Send + Sync,
{
    /// Get a value by key
    async fn get(&self, key: &str) -> Option<V>;

    /// Set a value for a key (overwrites if exists)
    async fn set(&self, key: &str, value: V);

    /// Remove a value by key, returns true if it existed
    async fn remove(&self, key: &str) -> bool;

    /// List all keys in the store
    async fn keys(&self) -> Vec<String>;
}

/// In-memory data store implementation using HashMap
#[derive(Clone)]
pub struct InMemStore<V>
where
    V: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone,
{
    data: Arc<Mutex<HashMap<String, V>>>,
}

impl<V> InMemStore<V>
where
    V: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone,
{
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl<V> Default for InMemStore<V>
where
    V: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl<V> DataStore<V> for InMemStore<V>
where
    V: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone,
{
    async fn get(&self, key: &str) -> Option<V> {
        let data_guard = self.data.lock().await;
        data_guard.get(key).cloned()
    }

    async fn set(&self, key: &str, value: V) {
        let mut data_guard = self.data.lock().await;
        data_guard.insert(key.to_string(), value);
    }

    async fn remove(&self, key: &str) -> bool {
        let mut data_guard = self.data.lock().await;
        data_guard.remove(key).is_some()
    }

    async fn keys(&self) -> Vec<String> {
        let data_guard = self.data.lock().await;
        data_guard.keys().cloned().collect()
    }
}

/// Filesystem-based YAML data store
/// Each key is stored as a separate .yaml file in the specified directory
#[derive(Clone)]
pub struct FilesystemYamlStore<V>
where
    V: Serialize + for<'de> Deserialize<'de> + Send + Sync,
{
    storage_dir: PathBuf,
    // In-memory cache for loaded values
    cache: Arc<Mutex<HashMap<String, V>>>,
    // Track which keys have been loaded from disk
    loaded_keys: Arc<Mutex<HashMap<String, bool>>>,
    _phantom: PhantomData<V>,
}

impl<V> FilesystemYamlStore<V>
where
    V: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone,
{
    pub fn new(storage_dir: PathBuf) -> Self {
        Self {
            storage_dir,
            cache: Arc::new(Mutex::new(HashMap::new())),
            loaded_keys: Arc::new(Mutex::new(HashMap::new())),
            _phantom: PhantomData,
        }
    }

    /// Get the file path for a key
    fn get_file_path(&self, key: &str) -> PathBuf {
        let safe_filename = encode_key_to_filename(key);
        self.storage_dir.join(format!("{}.yaml", safe_filename))
    }

    /// Load value from disk for a specific key
    async fn load_from_disk(&self, key: &str) -> Option<V> {
        let file_path = self.get_file_path(key);

        match fs::read_to_string(&file_path).await {
            Ok(content) => serde_yaml::from_str::<V>(&content).ok(),
            Err(_) => None, // File doesn't exist or can't be read
        }
    }

    /// Save value to disk for a specific key
    async fn save_to_disk(&self, key: &str, value: &V) -> Result<(), std::io::Error> {
        // Create directory if it doesn't exist
        fs::create_dir_all(&self.storage_dir).await?;

        let file_path = self.get_file_path(key);

        match serde_yaml::to_string(value) {
            Ok(content) => fs::write(&file_path, content).await,
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to serialize to YAML: {}", e),
            )),
        }
    }

    /// Ensure a value is loaded for a key (lazy loading)
    async fn ensure_loaded(&self, key: &str) {
        let loaded_guard = self.loaded_keys.lock().await;
        if loaded_guard.get(key).copied().unwrap_or(false) {
            // Already loaded
            return;
        }
        drop(loaded_guard); // Release lock while doing I/O

        // Load from disk
        if let Some(value) = self.load_from_disk(key).await {
            let mut cache_guard = self.cache.lock().await;
            cache_guard.insert(key.to_string(), value);
        }

        // Mark as loaded (even if file didn't exist)
        let mut loaded_guard = self.loaded_keys.lock().await;
        loaded_guard.insert(key.to_string(), true);
    }

    /// Delete file from disk
    async fn delete_from_disk(&self, key: &str) -> Result<(), std::io::Error> {
        let file_path = self.get_file_path(key);
        fs::remove_file(&file_path).await
    }
}

#[async_trait::async_trait]
impl<V> DataStore<V> for FilesystemYamlStore<V>
where
    V: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone,
{
    async fn get(&self, key: &str) -> Option<V> {
        self.ensure_loaded(key).await;
        let cache_guard = self.cache.lock().await;
        cache_guard.get(key).cloned()
    }

    async fn set(&self, key: &str, value: V) {
        // Update cache
        let mut cache_guard = self.cache.lock().await;
        cache_guard.insert(key.to_string(), value.clone());
        drop(cache_guard);

        // Mark as loaded
        let mut loaded_guard = self.loaded_keys.lock().await;
        loaded_guard.insert(key.to_string(), true);
        drop(loaded_guard);

        // Save to disk (ignore errors for now - could log them)
        let _ = self.save_to_disk(key, &value).await;
    }

    async fn remove(&self, key: &str) -> bool {
        self.ensure_loaded(key).await;

        // Remove from cache
        let mut cache_guard = self.cache.lock().await;
        let existed = cache_guard.remove(key).is_some();
        drop(cache_guard);

        if existed {
            // Delete from disk (ignore errors)
            let _ = self.delete_from_disk(key).await;
        }

        existed
    }

    async fn keys(&self) -> Vec<String> {
        // For filesystem store, list all .yaml files in the directory
        match fs::read_dir(&self.storage_dir).await {
            Ok(mut entries) => {
                let mut keys = Vec::new();
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if let Some(file_name) = entry.file_name().to_str() && file_name.ends_with(".yaml") {
                        let encoded_key = file_name.trim_end_matches(".yaml");
                        let decoded_key = decode_filename_to_key(encoded_key);
                        keys.push(decoded_key);
                    }
                }
                keys
            }
            Err(_) => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestData {
        value: String,
        count: i32,
    }

    #[tokio::test]
    async fn test_inmem_store_set_and_get() {
        let store = InMemStore::<TestData>::new();
        let data = TestData {
            value: "test".to_string(),
            count: 42,
        };

        store.set("key1", data.clone()).await;
        let retrieved = store.get("key1").await;

        assert_eq!(retrieved, Some(data));
    }

    #[tokio::test]
    async fn test_inmem_store_remove() {
        let store = InMemStore::<TestData>::new();
        let data = TestData {
            value: "test".to_string(),
            count: 42,
        };

        store.set("key1", data.clone()).await;
        assert_eq!(store.get("key1").await, Some(data));

        let removed = store.remove("key1").await;
        assert!(removed);
        assert_eq!(store.get("key1").await, None);

        let removed_again = store.remove("key1").await;
        assert!(!removed_again);
    }

    #[tokio::test]
    async fn test_inmem_store_keys() {
        let store = InMemStore::<TestData>::new();

        store
            .set(
                "key1",
                TestData {
                    value: "test1".to_string(),
                    count: 1,
                },
            )
            .await;
        store
            .set(
                "key2",
                TestData {
                    value: "test2".to_string(),
                    count: 2,
                },
            )
            .await;

        let keys = store.keys().await;
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
    }

    #[tokio::test]
    async fn test_filesystem_store_set_and_get() {
        let temp_dir = std::env::temp_dir().join("yoroolbot_test_fs_store");
        let _ = fs::remove_dir_all(&temp_dir).await; // Clean up if exists
        let store = FilesystemYamlStore::<TestData>::new(temp_dir.clone());

        let data = TestData {
            value: "test".to_string(),
            count: 42,
        };

        store.set("key1", data.clone()).await;
        let retrieved = store.get("key1").await;

        assert_eq!(retrieved, Some(data));

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_filesystem_store_persistence() {
        let temp_dir = std::env::temp_dir().join("yoroolbot_test_fs_persistence");
        let _ = fs::remove_dir_all(&temp_dir).await; // Clean up if exists

        let data = TestData {
            value: "test".to_string(),
            count: 42,
        };

        // Create store and set value
        {
            let store = FilesystemYamlStore::<TestData>::new(temp_dir.clone());
            store.set("key1", data.clone()).await;
        }

        // Create new store instance and verify value persisted
        {
            let store = FilesystemYamlStore::<TestData>::new(temp_dir.clone());
            let retrieved = store.get("key1").await;
            assert_eq!(retrieved, Some(data));
        }

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_filesystem_store_remove() {
        let temp_dir = std::env::temp_dir().join("yoroolbot_test_fs_remove");
        let _ = fs::remove_dir_all(&temp_dir).await; // Clean up if exists
        let store = FilesystemYamlStore::<TestData>::new(temp_dir.clone());

        let data = TestData {
            value: "test".to_string(),
            count: 42,
        };

        store.set("key1", data.clone()).await;
        assert_eq!(store.get("key1").await, Some(data));

        let removed = store.remove("key1").await;
        assert!(removed);
        assert_eq!(store.get("key1").await, None);

        // Verify file was deleted
        let file_path = temp_dir.join("key1.yaml");
        assert!(!file_path.exists());

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_filesystem_store_with_encoded_keys() {
        let temp_dir = std::env::temp_dir().join("yoroolbot_test_fs_encoded");
        let _ = fs::remove_dir_all(&temp_dir).await; // Clean up if exists
        let store = FilesystemYamlStore::<TestData>::new(temp_dir.clone());

        // Test keys with forbidden characters
        let test_cases = vec![
            ("path/to/key", "path%2Fto%2Fkey.yaml"),
            ("key:value", "key%3Avalue.yaml"),
            (".hidden", "%2Ehidden.yaml"),
            ("file*.txt", "file%2A.txt.yaml"),
            ("space key", "space%20key.yaml"),
        ];

        for (key, expected_filename) in test_cases {
            let data = TestData {
                value: format!("data for {}", key),
                count: 1,
            };

            // Set the value
            store.set(key, data.clone()).await;

            // Verify the file was created with encoded filename
            let file_path = temp_dir.join(expected_filename);
            assert!(
                file_path.exists(),
                "File {:?} should exist for key '{}'",
                file_path,
                key
            );

            // Retrieve the value
            let retrieved = store.get(key).await;
            assert_eq!(retrieved, Some(data.clone()));
        }

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_filesystem_store_keys_with_encoding() {
        let temp_dir = std::env::temp_dir().join("yoroolbot_test_fs_keys_encoded");
        let _ = fs::remove_dir_all(&temp_dir).await; // Clean up if exists
        let store = FilesystemYamlStore::<TestData>::new(temp_dir.clone());

        // Store values with various keys including forbidden chars
        let keys = vec!["simple", "path/to/key", "key:value", ".hidden", "space key"];

        for key in &keys {
            store
                .set(
                    key,
                    TestData {
                        value: format!("data for {}", key),
                        count: 1,
                    },
                )
                .await;
        }

        // Retrieve all keys
        let retrieved_keys = store.keys().await;

        // Verify all keys are decoded correctly
        assert_eq!(retrieved_keys.len(), keys.len());
        for key in keys {
            assert!(
                retrieved_keys.contains(&key.to_string()),
                "Key '{}' should be in retrieved keys",
                key
            );
        }

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_filesystem_store_round_trip_complex_keys() {
        let temp_dir = std::env::temp_dir().join("yoroolbot_test_fs_complex");
        let _ = fs::remove_dir_all(&temp_dir).await; // Clean up if exists

        // Test with complex keys that have multiple forbidden characters
        let complex_key = "path/to:key*.txt with spaces";
        let data = TestData {
            value: "complex data".to_string(),
            count: 99,
        };

        // Create store and set value
        {
            let store = FilesystemYamlStore::<TestData>::new(temp_dir.clone());
            store.set(complex_key, data.clone()).await;
        }

        // Create new store instance and verify value persisted with correct key
        {
            let store = FilesystemYamlStore::<TestData>::new(temp_dir.clone());
            let retrieved = store.get(complex_key).await;
            assert_eq!(retrieved, Some(data.clone()));

            // Verify the key appears in keys() list
            let keys = store.keys().await;
            assert!(keys.contains(&complex_key.to_string()));
        }

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir).await;
    }
}
