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
        let parts: Vec<&str> = key.splitn(2, '.').collect();
        match parts[0] {
            "data_dir" => Some(self.data_dir.clone()),
            "editor" => Some(self.editor()),
            "theme" => parts.get(1).and_then(|k| self.theme.get(*k)),
            "keybindings" => parts.get(1).and_then(|k| self.keybindings.get(*k)),
            _ => None,
        }
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        let parts: Vec<&str> = key.splitn(2, '.').collect();
        match parts[0] {
            "data_dir" => self.data_dir = value.to_string(),
            "editor" => self.editor = value.to_string(),
            "theme" => {
                let k = parts.get(1).ok_or_else(|| AppError::Config("Missing theme key".into()))?;
                self.theme.set(k, value)?;
            }
            "keybindings" => {
                let k = parts.get(1).ok_or_else(|| AppError::Config("Missing keybindings key".into()))?;
                self.keybindings.set(k, value)?;
            }
            _ => {
                return Err(AppError::Config(format!(
                    "Unknown config key: '{}'. Valid: data_dir, editor, theme.<field>, keybindings.<field>",
                    key
                )))
            }
        }
        self.save()?;
        Ok(())
    }
}

impl ThemeConfig {
    fn get(&self, key: &str) -> Option<String> {
        match key {
            "selected_bg" => Some(self.selected_bg.clone()),
            "done_fg" => Some(self.done_fg.clone()),
            "border" => Some(self.border.clone()),
            "command_bar_bg" => Some(self.command_bar_bg.clone()),
            "modal_bg" => Some(self.modal_bg.clone()),
            "title_fg" => Some(self.title_fg.clone()),
            "normal_bg" => Some(self.normal_bg.clone()),
            "status_bar_fg" => Some(self.status_bar_fg.clone()),
            _ => None,
        }
    }

    fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "selected_bg" => self.selected_bg = value.to_string(),
            "done_fg" => self.done_fg = value.to_string(),
            "border" => self.border = value.to_string(),
            "command_bar_bg" => self.command_bar_bg = value.to_string(),
            "modal_bg" => self.modal_bg = value.to_string(),
            "title_fg" => self.title_fg = value.to_string(),
            "normal_bg" => self.normal_bg = value.to_string(),
            "status_bar_fg" => self.status_bar_fg = value.to_string(),
            _ => return Err(AppError::Config(format!("Unknown theme key: '{}'", key))),
        }
        Ok(())
    }
}

impl KeybindingsConfig {
    fn get(&self, key: &str) -> Option<String> {
        match key {
            "move_down" => Some(self.move_down.clone()),
            "move_up" => Some(self.move_up.clone()),
            "toggle_done" => Some(self.toggle_done.clone()),
            "show_detail" => Some(self.show_detail.clone()),
            "filter_all" => Some(self.filter_all.clone()),
            "filter_todo" => Some(self.filter_todo.clone()),
            "filter_done" => Some(self.filter_done.clone()),
            "new_task" => Some(self.new_task.clone()),
            "edit_task" => Some(self.edit_task.clone()),
            "delete_task" => Some(self.delete_task.clone()),
            "open_config" => Some(self.open_config.clone()),
            "command_bar" => Some(self.command_bar.clone()),
            "help" => Some(self.help.clone()),
            "reload" => Some(self.reload.clone()),
            "quit" => Some(self.quit.clone()),
            _ => None,
        }
    }

    fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "move_down" => self.move_down = value.to_string(),
            "move_up" => self.move_up = value.to_string(),
            "toggle_done" => self.toggle_done = value.to_string(),
            "show_detail" => self.show_detail = value.to_string(),
            "filter_all" => self.filter_all = value.to_string(),
            "filter_todo" => self.filter_todo = value.to_string(),
            "filter_done" => self.filter_done = value.to_string(),
            "new_task" => self.new_task = value.to_string(),
            "edit_task" => self.edit_task = value.to_string(),
            "delete_task" => self.delete_task = value.to_string(),
            "open_config" => self.open_config = value.to_string(),
            "command_bar" => self.command_bar = value.to_string(),
            "help" => self.help = value.to_string(),
            "reload" => self.reload = value.to_string(),
            "quit" => self.quit = value.to_string(),
            _ => return Err(AppError::Config(format!("Unknown keybindings key: '{}'", key))),
        }
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
