use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::errors::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub data_dir: String,
    pub editor: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            data_dir: "~/.local/share/easytodo/tasks".into(),
            editor: String::new(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Config> {
        let config_path = Self::config_path()?;
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            toml::from_str(&content).map_err(|e| AppError::Toml(e.to_string()))
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }

    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().ok_or(AppError::HomeDir)?;
        Ok(config_dir.join("easytodo").join("config.toml"))
    }

    pub fn resolved_data_dir(&self) -> Result<PathBuf> {
        let path = if self.data_dir.starts_with('~') {
            let home = dirs::home_dir().ok_or(AppError::HomeDir)?;
            home.join(&self.data_dir[2..])
        } else {
            PathBuf::from(&self.data_dir)
        };
        Ok(path)
    }

    pub fn editor(&self) -> String {
        if !self.editor.is_empty() {
            self.editor.clone()
        } else if let Ok(editor) = std::env::var("EDITOR") {
            editor
        } else {
            "vim".into()
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "data_dir" => Some(self.data_dir.clone()),
            "editor" => Some(self.editor()),
            _ => None,
        }
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "data_dir" => self.data_dir = value.to_string(),
            "editor" => self.editor = value.to_string(),
            _ => {
                return Err(AppError::Config(format!(
                    "Unknown config key: {}. Valid keys: data_dir, editor",
                    key
                )))
            }
        }
        self.save()?;
        Ok(())
    }
}
