use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryItem {
    pub title: String,
    pub url: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkItem {
    pub title: String,
    pub url: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppData {
    pub dark_mode: bool,
    pub history: Vec<HistoryItem>,
    pub bookmarks: Vec<BookmarkItem>,
}

pub struct StorageManager {
    file_path: PathBuf,
    pub data: AppData,
}

impl StorageManager {
    pub fn new() -> Self {
        let file_path = Self::get_storage_path();
        let data = Self::load_from_path(&file_path).unwrap_or_default();
        Self { file_path, data }
    }

    fn get_storage_path() -> PathBuf {
        if let Some(mut config_dir) = dirs::config_dir() {
            config_dir.push("midlevel");
            let _ = fs::create_dir_all(&config_dir);
            config_dir.push("data.json");
            config_dir
        } else {
            PathBuf::from("midlevel_data.json")
        }
    }

    fn load_from_path(path: &PathBuf) -> Option<AppData> {
        let content = fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn save(&self) {
        if self.file_path.as_os_str().is_empty() {
            return;
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.data) {
            let _ = fs::write(&self.file_path, json);
        }
    }

    pub fn add_history(&mut self, title: String, url: String) {
        if url.is_empty() || url.starts_with("data:") || url == "about:blank" {
            return;
        }

        if let Some(last) = self.data.history.last() {
            if last.url == url {
                return;
            }
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.data.history.push(HistoryItem { title, url, timestamp });

        if self.data.history.len() > 1000 {
            self.data.history.remove(0);
        }

        self.save();
    }

    pub fn is_bookmarked(&self, url: &str) -> bool {
        self.data.bookmarks.iter().any(|b| b.url == url)
    }

    pub fn toggle_bookmark(&mut self, title: String, url: String) -> bool {
        if url.is_empty() || url.starts_with("data:") || url == "about:blank" {
            return false;
        }

        if let Some(idx) = self.data.bookmarks.iter().position(|b| b.url == url) {
            self.data.bookmarks.remove(idx);
            self.save();
            false
        } else {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);

            self.data.bookmarks.push(BookmarkItem { title, url, timestamp });
            self.save();
            true
        }
    }

    pub fn toggle_dark_mode(&mut self) -> bool {
        self.data.dark_mode = !self.data.dark_mode;
        self.save();
        self.data.dark_mode
    }
}