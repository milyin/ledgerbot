//
// "Period" is just a set of commands defining full expense information for a specified named period
// It contains configuration commands and expense commands
// Each "Period" is stored in the text file named "<period_name>.txt" in the specified storage directory
//

use std::path::{Path, PathBuf};

use teloxide::types::ChatId;

const CURRENT_PERIOD_NAME: &str = "__current_period__";

#[async_trait::async_trait]
pub trait PeriodsStorageTrait: Send + Sync {
    /// List all stored periods
    async fn list_periods(&self, chat_id: ChatId) -> Vec<String>;

    /// Load period by name
    async fn load_period(&self, chat_id: ChatId, period_name: &str) -> Option<Vec<String>>;

    /// Save period by name
    async fn save_period(&self, chat_id: ChatId, period_name: &str, data: Vec<String>);
}

pub struct PeriodsStorage {
    periods: tokio::sync::Mutex<
        std::collections::HashMap<ChatId, std::collections::HashMap<String, Vec<String>>>,
    >,
}

impl PeriodsStorage {
    pub fn new() -> Self {
        Self {
            periods: tokio::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl PeriodsStorageTrait for PeriodsStorage {
    async fn list_periods(&self, chat_id: ChatId) -> Vec<String> {
        let periods_guard = self.periods.lock().await;
        periods_guard
            .get(&chat_id)
            .map(|chat_periods| chat_periods.keys().cloned().collect())
            .unwrap_or_default()
    }

    async fn load_period(&self, chat_id: ChatId, period_name: &str) -> Option<Vec<String>> {
        let periods_guard = self.periods.lock().await;
        periods_guard
            .get(&chat_id)
            .and_then(|chat_periods| chat_periods.get(period_name))
            .cloned()
    }

    async fn save_period(&self, chat_id: ChatId, period_name: &str, data: Vec<String>) {
        let mut periods_guard = self.periods.lock().await;
        periods_guard
            .entry(chat_id)
            .or_default()
            .insert(period_name.to_string(), data);
    }
}

pub struct PeriodsFileStorage {
    storage_dir: PathBuf,
}

impl PeriodsFileStorage {
    pub fn new(storage_dir: impl AsRef<Path>) -> Self {
        Self {
            storage_dir: storage_dir.as_ref().to_path_buf(),
        }
    }

    fn chat_dir(&self, chat_id: ChatId) -> PathBuf {
        self.storage_dir.join(chat_id.to_string())
    }

    fn period_file_path(&self, chat_id: ChatId, period_name: &str) -> PathBuf {
        self.chat_dir(chat_id).join(format!("{}.txt", period_name))
    }
}

#[async_trait::async_trait]
impl PeriodsStorageTrait for PeriodsFileStorage {
    async fn list_periods(&self, chat_id: ChatId) -> Vec<String> {
        let mut periods = Vec::new();
        if let Ok(entries) = tokio::fs::read_dir(self.chat_dir(chat_id)).await {
            let mut dir_entries = entries;
            while let Ok(Some(entry)) = dir_entries.next_entry().await {
                if let Some(file_name) = entry.file_name().to_str() {
                    // skip CURRENT_PERIOD_NAME file
                    if file_name == CURRENT_PERIOD_NAME {
                        continue;
                    }
                    // only consider .txt files
                    if let Some(period_name) = file_name.strip_suffix(".txt") {
                        periods.push(period_name.to_string());
                    }
                }
            }
        }
        periods
    }

    async fn load_period(&self, chat_id: ChatId, period_name: &str) -> Option<Vec<String>> {
        let file_path = self.period_file_path(chat_id, period_name);
        log::info!("Loading period from file: {:?}", file_path);
        match tokio::fs::read_to_string(file_path).await {
            Ok(content) => Some(content.lines().map(|line| line.to_string()).collect()),
            Err(_) => None,
        }
    }

    async fn save_period(&self, chat_id: ChatId, period_name: &str, data: Vec<String>) {
        let file_path = self.period_file_path(chat_id, period_name);
        log::info!("Saving period to file: {:?}", file_path);
        let content = data.join("\n");
        // make sure the chat directory exists
        let chat_dir = self.chat_dir(chat_id);
        let _ = tokio::fs::create_dir_all(chat_dir).await;
        let _ = tokio::fs::write(file_path, content).await;
    }
}
