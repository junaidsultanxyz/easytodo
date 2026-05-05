use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::errors::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub data_dir: String,
    pub editor: String,
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub keybindings: KeybindingsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            data_dir: "~/.local/share/easytodo/tasks".into(),
            editor: String::new(),
            theme: ThemeConfig::default(),
            keybindings: KeybindingsConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    #[serde(default = "default_selected_bg")]
    pub selected_bg: String,
    #[serde(default = "default_done_fg")]
    pub done_fg: String,
    #[serde(default = "default_border")]
    pub border: String,
    #[serde(default = "default_command_bar_bg")]
    pub command_bar_bg: String,
    #[serde(default = "default_modal_bg")]
    pub modal_bg: String,
    #[serde(default = "default_title_fg")]
    pub title_fg: String,
    #[serde(default = "default_normal_bg")]
    pub normal_bg: String,
    #[serde(default = "default_status_bar_fg")]
    pub status_bar_fg: String,
}

fn default_selected_bg() -> String { "rgb(60,60,80)".into() }
fn default_done_fg() -> String { "rgb(100,140,100)".into() }
fn default_border() -> String { "rgb(80,80,120)".into() }
fn default_command_bar_bg() -> String { "rgb(30,30,50)".into() }
fn default_modal_bg() -> String { "rgb(25,25,45)".into() }
fn default_title_fg() -> String { "rgb(180,180,220)".into() }
fn default_normal_bg() -> String { "rgb(20,20,35)".into() }
fn default_status_bar_fg() -> String { "rgb(130,130,160)".into() }

impl Default for ThemeConfig {
    fn default() -> Self {
        ThemeConfig {
            selected_bg: default_selected_bg(),
            done_fg: default_done_fg(),
            border: default_border(),
            command_bar_bg: default_command_bar_bg(),
            modal_bg: default_modal_bg(),
            title_fg: default_title_fg(),
            normal_bg: default_normal_bg(),
            status_bar_fg: default_status_bar_fg(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KeybindingsConfig {
    #[serde(default = "default_move_down")]
    pub move_down: String,
    #[serde(default = "default_move_up")]
    pub move_up: String,
    #[serde(default = "default_toggle_done")]
    pub toggle_done: String,
    #[serde(default = "default_show_detail")]
    pub show_detail: String,
    #[serde(default = "default_filter_all")]
    pub filter_all: String,
    #[serde(default = "default_filter_todo")]
    pub filter_todo: String,
    #[serde(default = "default_filter_done")]
    pub filter_done: String,
    #[serde(default = "default_new_task")]
    pub new_task: String,
    #[serde(default = "default_edit_task")]
    pub edit_task: String,
    #[serde(default = "default_delete_task")]
    pub delete_task: String,
    #[serde(default = "default_open_config")]
    pub open_config: String,
    #[serde(default = "default_command_bar")]
    pub command_bar: String,
    #[serde(default = "default_help")]
    pub help: String,
    #[serde(default = "default_reload")]
    pub reload: String,
    #[serde(default = "default_quit")]
    pub quit: String,
}

fn default_move_down() -> String { "j".into() }
fn default_move_up() -> String { "k".into() }
fn default_toggle_done() -> String { "Enter".into() }
fn default_show_detail() -> String { "l".into() }
fn default_filter_all() -> String { "1".into() }
fn default_filter_todo() -> String { "2".into() }
fn default_filter_done() -> String { "3".into() }
fn default_new_task() -> String { "Ctrl+N".into() }
fn default_edit_task() -> String { "Ctrl+E".into() }
fn default_delete_task() -> String { "Ctrl+D".into() }
fn default_open_config() -> String { "Ctrl+B".into() }
fn default_command_bar() -> String { "Ctrl+P".into() }
fn default_help() -> String { "Ctrl+H".into() }
fn default_reload() -> String { "Ctrl+R".into() }
fn default_quit() -> String { "Ctrl+Q".into() }

impl Default for KeybindingsConfig {
    fn default() -> Self {
        KeybindingsConfig {
            move_down: default_move_down(),
            move_up: default_move_up(),
            toggle_done: default_toggle_done(),
            show_detail: default_show_detail(),
            filter_all: default_filter_all(),
            filter_todo: default_filter_todo(),
            filter_done: default_filter_done(),
            new_task: default_new_task(),
            edit_task: default_edit_task(),
            delete_task: default_delete_task(),
            open_config: default_open_config(),
            command_bar: default_command_bar(),
            help: default_help(),
            reload: default_reload(),
            quit: default_quit(),
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

pub fn parse_color(s: &str) -> Option<(u8, u8, u8)> {
    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some((r, g, b));
        }
    }
    if let Some(inner) = s.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 3 {
            let r = parts[0].trim().parse().ok()?;
            let g = parts[1].trim().parse().ok()?;
            let b = parts[2].trim().parse().ok()?;
            return Some((r, g, b));
        }
    }
    None
}
