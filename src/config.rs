//! Configuration management
//!
//! Handles loading, saving, and managing application settings.

use std::path::PathBuf;
use std::fs;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Whether to start with the system
    pub start_with_system: bool,
    
    /// Whether to start minimized to tray
    pub start_minimized: bool,
    
    /// Whether to show notifications
    pub show_notifications: bool,
    
    /// Whether to play sounds on transformation
    pub play_sounds: bool,
    
    /// Clipboard polling interval in milliseconds
    pub poll_interval_ms: u64,
    
    /// Whether to enable automatic transformation
    pub auto_transform: bool,
    
    /// Global hotkey to toggle transformation
    pub toggle_hotkey: Option<String>,
    
    /// Global hotkey to open quick menu
    pub quick_menu_hotkey: Option<String>,
    
    /// Global hotkey to open dashboard
    pub dashboard_hotkey: Option<String>,
    
    /// Theme: "dark", "light", or "system"
    pub theme: String,
    
    /// Keep clipboard history
    pub keep_history: bool,
    
    /// Maximum history size
    pub max_history_size: usize,
    
    /// ID of the currently active recipe (UUID as string)
    pub active_recipe_id: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            start_with_system: false,
            start_minimized: true,
            show_notifications: true,
            play_sounds: false,
            poll_interval_ms: 250,
            auto_transform: true,
            toggle_hotkey: Some("Ctrl+Shift+T".into()),
            quick_menu_hotkey: Some("Ctrl+Shift+V".into()),
            dashboard_hotkey: Some("Ctrl+Shift+D".into()),
            theme: "system".into(),
            keep_history: true,
            max_history_size: 100,
            active_recipe_id: None,
        }
    }
}

impl Config {
    /// Get the config file path
    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to find config directory")?
            .join("9paste");
        
        fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;
        
        Ok(config_dir.join("config.json"))
    }
    
    /// Load config from disk or create default
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        
        if path.exists() {
            let data = fs::read_to_string(&path)
                .context("Failed to read config file")?;
            serde_json::from_str(&data)
                .context("Failed to parse config file")
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }
    
    /// Save config to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let data = serde_json::to_string_pretty(self)
            .context("Failed to serialize config")?;
        fs::write(&path, data)
            .context("Failed to write config file")?;
        Ok(())
    }
    
    /// Get the data directory path
    pub fn data_dir() -> Result<PathBuf> {
        let data_dir = dirs::data_dir()
            .context("Failed to find data directory")?
            .join("9paste");
        
        fs::create_dir_all(&data_dir)
            .context("Failed to create data directory")?;
        
        Ok(data_dir)
    }
    
    /// Get the config directory path
    pub fn config_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to find config directory")?
            .join("9paste");
        
        fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;
        
        Ok(config_dir)
    }
}

/// Clipboard history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Original text before transformation
    pub original: String,
    /// Transformed text (if any)
    pub transformed: Option<String>,
    /// Recipe ID used for transformation
    pub recipe_id: Option<String>,
    /// Recipe name used for transformation
    pub recipe_name: Option<String>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Clipboard history manager
pub struct HistoryManager {
    entries: Vec<HistoryEntry>,
    max_size: usize,
    history_path: PathBuf,
}

impl HistoryManager {
    /// Create a new history manager
    pub fn new(max_size: usize) -> Result<Self> {
        let history_path = Config::data_dir()?.join("history.json");
        
        let entries = if history_path.exists() {
            let data = fs::read_to_string(&history_path)
                .context("Failed to read history file")?;
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Vec::new()
        };
        
        Ok(Self {
            entries,
            max_size,
            history_path,
        })
    }
    
    /// Add an entry to history
    pub fn add(&mut self, entry: HistoryEntry) -> Result<()> {
        self.entries.insert(0, entry);
        
        // Trim to max size
        if self.entries.len() > self.max_size {
            self.entries.truncate(self.max_size);
        }
        
        self.save()
    }
    
    /// Get all history entries
    pub fn get_all(&self) -> &[HistoryEntry] {
        &self.entries
    }
    
    /// Clear all history
    pub fn clear(&mut self) -> Result<()> {
        self.entries.clear();
        self.save()
    }
    
    /// Save history to disk
    fn save(&self) -> Result<()> {
        let data = serde_json::to_string_pretty(&self.entries)
            .context("Failed to serialize history")?;
        fs::write(&self.history_path, data)
            .context("Failed to write history file")?;
        Ok(())
    }
}
