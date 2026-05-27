use std::{collections::HashMap, marker::PhantomData, path::PathBuf, sync::Arc};

use serde::{Deserialize, Serialize};
use teloxide::types::ChatId;
use tokio::{fs, sync::Mutex};

/// Trait for key-value data storage with serializable values.
#[async_trait::async_trait]
pub trait DataStoreTrait<V>: Send + Sync
where
    V: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone,
{
    async fn get(&self, chat_id: ChatId, key: &str) -> Option<V>;
    async fn set(&self, chat_id: ChatId, key: &str, value: V);
    async fn remove(&self, chat_id: ChatId, key: &str) -> bool;
    async fn keys(&self, chat_id: ChatId) -> Vec<String>;
}

/// In-memory data store implementation using HashMap.
#[derive(Clone)]
pub struct InMemStore<V>
where
    V: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone,
{
    data: Arc<Mutex<HashMap<ChatId, HashMap<String, V>>>>,
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
impl<V> DataStoreTrait<V> for InMemStore<V>
where
    V: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone,
{
    async fn get(&self, chat_id: ChatId, key: &str) -> Option<V> {
        let data_guard = self.data.lock().await;
        let chat_data = data_guard.get(&chat_id)?;
        chat_data.get(key).cloned()
    }

    async fn set(&self, chat_id: ChatId, key: &str, value: V) {
        let mut data_guard = self.data.lock().await;
        let chat_data = data_guard.entry(chat_id).or_insert_with(HashMap::new);
        chat_data.insert(key.to_string(), value);
    }

    async fn remove(&self, chat_id: ChatId, key: &str) -> bool {
        let mut data_guard = self.data.lock().await;
        if let Some(chat_data) = data_guard.get_mut(&chat_id) {
            chat_data.remove(key).is_some()
        } else {
            false
        }
    }

    async fn keys(&self, chat_id: ChatId) -> Vec<String> {
        let data_guard = self.data.lock().await;
        data_guard
            .get(&chat_id)
            .map(|chat_data| chat_data.keys().cloned().collect())
            .unwrap_or_default()
    }
}

/// Filesystem-based YAML data store.
#[derive(Clone)]
pub struct FilesystemYamlStore<V>
where
    V: Serialize + for<'de> Deserialize<'de> + Send + Sync,
{
    storage_dir: PathBuf,
    cache: Arc<Mutex<HashMap<ChatId, HashMap<String, V>>>>,
    loaded_keys: Arc<Mutex<HashMap<ChatId, HashMap<String, bool>>>>,
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

    fn get_chat_dir(&self, chat_id: ChatId) -> PathBuf {
        let chat_id_str = chat_id.0.to_string();
        let safe_chat_dir = encode_key_to_filename(&chat_id_str);
        self.storage_dir.join(safe_chat_dir)
    }

    fn get_file_path(&self, chat_id: ChatId, key: &str) -> PathBuf {
        let safe_filename = encode_key_to_filename(key);
        self.get_chat_dir(chat_id)
            .join(format!("{}.yaml", safe_filename))
    }

    async fn load_from_disk(&self, chat_id: ChatId, key: &str) -> Option<V> {
        let file_path = self.get_file_path(chat_id, key);
        match fs::read_to_string(&file_path).await {
            Ok(content) => serde_yaml::from_str::<V>(&content).ok(),
            Err(_) => None,
        }
    }

    async fn save_to_disk(
        &self,
        chat_id: ChatId,
        key: &str,
        value: &V,
    ) -> Result<(), std::io::Error> {
        let chat_dir = self.get_chat_dir(chat_id);
        fs::create_dir_all(&chat_dir).await?;

        let file_path = self.get_file_path(chat_id, key);
        match serde_yaml::to_string(value) {
            Ok(content) => fs::write(&file_path, content).await,
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to serialize to YAML: {}", e),
            )),
        }
    }

    async fn ensure_loaded(&self, chat_id: ChatId, key: &str) {
        let loaded_guard = self.loaded_keys.lock().await;
        let is_loaded = loaded_guard
            .get(&chat_id)
            .and_then(|chat_keys| chat_keys.get(key).copied())
            .unwrap_or(false);
        if is_loaded {
            return;
        }
        drop(loaded_guard);

        if let Some(value) = self.load_from_disk(chat_id, key).await {
            let mut cache_guard = self.cache.lock().await;
            let chat_cache = cache_guard.entry(chat_id).or_insert_with(HashMap::new);
            chat_cache.insert(key.to_string(), value);
        }

        let mut loaded_guard = self.loaded_keys.lock().await;
        let chat_loaded = loaded_guard.entry(chat_id).or_insert_with(HashMap::new);
        chat_loaded.insert(key.to_string(), true);
    }

    async fn delete_from_disk(&self, chat_id: ChatId, key: &str) -> Result<(), std::io::Error> {
        let file_path = self.get_file_path(chat_id, key);
        fs::remove_file(&file_path).await
    }
}

#[async_trait::async_trait]
impl<V> DataStoreTrait<V> for FilesystemYamlStore<V>
where
    V: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone,
{
    async fn get(&self, chat_id: ChatId, key: &str) -> Option<V> {
        self.ensure_loaded(chat_id, key).await;
        let cache_guard = self.cache.lock().await;
        cache_guard
            .get(&chat_id)
            .and_then(|chat_cache| chat_cache.get(key).cloned())
    }

    async fn set(&self, chat_id: ChatId, key: &str, value: V) {
        let mut cache_guard = self.cache.lock().await;
        let chat_cache = cache_guard.entry(chat_id).or_insert_with(HashMap::new);
        chat_cache.insert(key.to_string(), value.clone());
        drop(cache_guard);

        let mut loaded_guard = self.loaded_keys.lock().await;
        let chat_loaded = loaded_guard.entry(chat_id).or_insert_with(HashMap::new);
        chat_loaded.insert(key.to_string(), true);
        drop(loaded_guard);

        if let Err(e) = self.save_to_disk(chat_id, key, &value).await {
            log::error!("Failed to save key '{}' for chat {}: {}", key, chat_id.0, e);
        }
    }

    async fn remove(&self, chat_id: ChatId, key: &str) -> bool {
        let mut removed = false;

        let mut cache_guard = self.cache.lock().await;
        if let Some(chat_cache) = cache_guard.get_mut(&chat_id) {
            removed = chat_cache.remove(key).is_some();
        }
        drop(cache_guard);

        let mut loaded_guard = self.loaded_keys.lock().await;
        if let Some(chat_loaded) = loaded_guard.get_mut(&chat_id) {
            chat_loaded.insert(key.to_string(), true);
        }
        drop(loaded_guard);

        match self.delete_from_disk(chat_id, key).await {
            Ok(()) => true,
            Err(_) => removed,
        }
    }

    async fn keys(&self, chat_id: ChatId) -> Vec<String> {
        let chat_dir = self.get_chat_dir(chat_id);
        let mut disk_keys = Vec::new();

        if let Ok(mut entries) = fs::read_dir(&chat_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Some(filename) = entry.file_name().to_str()
                    && let Some(name_without_ext) = filename.strip_suffix(".yaml")
                    && let Ok(decoded_key) = decode_filename_to_key(name_without_ext)
                {
                    disk_keys.push(decoded_key);
                }
            }
        }

        let cache_guard = self.cache.lock().await;
        let cache_keys: Vec<String> = cache_guard
            .get(&chat_id)
            .map(|chat_cache| chat_cache.keys().cloned().collect())
            .unwrap_or_default();
        drop(cache_guard);

        let mut all_keys = disk_keys;
        for key in cache_keys {
            if !all_keys.contains(&key) {
                all_keys.push(key);
            }
        }
        all_keys.sort();
        all_keys.dedup();
        all_keys
    }
}

pub fn encode_key_to_filename(key: &str) -> String {
    key.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' => char::from(b).to_string(),
            _ => format!("%{:02X}", b),
        })
        .collect()
}

pub fn decode_filename_to_key(encoded: &str) -> Result<String, String> {
    let mut bytes = Vec::new();
    let chars: Vec<char> = encoded.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '%' {
            if i + 2 >= chars.len() {
                return Err("Incomplete percent encoding".to_string());
            }
            let hex_str = format!("{}{}", chars[i + 1], chars[i + 2]);
            let byte = u8::from_str_radix(&hex_str, 16)
                .map_err(|_| format!("Invalid hex encoding: %{}", hex_str))?;
            bytes.push(byte);
            i += 3;
        } else {
            bytes.push(chars[i] as u8);
            i += 1;
        }
    }

    String::from_utf8(bytes).map_err(|e| format!("Invalid UTF-8: {}", e))
}
