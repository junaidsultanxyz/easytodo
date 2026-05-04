use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::Backend;
use ratatui::layout::{Constraint, Layout};
use ratatui::Terminal;

use easytodo::action::Action;
use easytodo::config::Config;
use easytodo::errors::{AppError, Result};
use easytodo::task::model::{Filter, Status, TaskField};
use easytodo::task::store::FsTaskStore;
use easytodo::task::store::TaskStore;

use crate::tui::command_bar::CommandBar;
use crate::tui::modal::ModalState;
use crate::tui::style::Theme;
use crate::tui::task_list::TaskList;
use crate::tui::Component;

struct WatchedFile {
    path: PathBuf,
    last_modified: SystemTime,
    action: WatchedFileAction,
}

enum WatchedFileAction {
    ReloadConfig,
    ReloadTasks,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Normal,
    Command,
    Modal,
}

pub struct App {
    pub mode: AppMode,
    pub store: FsTaskStore,
    pub config: Config,
    pub theme: Theme,
    pub tasks: Vec<easytodo::task::model::Task>,
    pub filter: Filter,
    pub task_list: TaskList,
    pub command_bar: CommandBar,
    pub modal: ModalState,
    pub status_message: Option<String>,
    pub status_message_time: Option<Instant>,
    watched_files: Vec<WatchedFile>,
    last_event_time: Instant,
    pub needs_clear: bool,
    pub should_quit: bool,
}

impl App {
    pub fn new(store: FsTaskStore, config: Config) -> Self {
        App {
            mode: AppMode::Normal,
            store,
            config,
            theme: Theme::default(),
            tasks: Vec::new(),
            filter: Filter::All,
            task_list: TaskList::new(),
            command_bar: CommandBar::new(),
            modal: ModalState::new(),
            status_message: None,
            status_message_time: None,
            watched_files: Vec::new(),
            last_event_time: Instant::now(),
            needs_clear: false,
            should_quit: false,
        }
    }

    pub fn set_status_message(&mut self, msg: String) {
        self.status_message = Some(msg);
        self.status_message_time = Some(Instant::now());
    }

    pub fn clear_expired_message(&mut self) {
        if let Some(time) = self.status_message_time {
            if !self.command_bar.visible && time.elapsed() >= Duration::from_secs(3) {
                self.status_message = None;
                self.status_message_time = None;
            }
        }
    }

    pub fn resolve_id(&self, id: &str) -> Result<String> {
        if id == "." {
            self.tasks
                .get(self.task_list.selected_index)
                .map(|t| t.id.clone())
                .ok_or_else(|| AppError::CommandError("No task selected".into()))
        } else {
            Ok(id.to_string())
        }
    }

    pub fn refresh_tasks(&mut self) -> Result<()> {
        self.tasks = self.store.list(self.filter)?;
        self.task_list.clamp_selection(self.tasks.len());
        Ok(())
    }

    pub fn handle_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::MoveUp => {
                self.task_list.selected_index = self.task_list.selected_index.saturating_sub(1);
            }
            Action::MoveDown => {
                if self.task_list.selected_index + 1 < self.tasks.len() {
                    self.task_list.selected_index += 1;
                }
            }
            Action::NewTask {
                title,
                description,
                due,
            } => {
                let task = self.store.create(&title, &description, due)?;
                self.refresh_tasks()?;
                self.set_status_message(format!("Task '{}' created (id: {})", task.title, task.id));
            }
            Action::NewTaskPrefill => {
                self.mode = AppMode::Command;
                self.command_bar.show_with_text("new ");
            }
            Action::EditTaskPrefill => {
                self.mode = AppMode::Command;
                self.command_bar.show_with_text("edit . ");
            }
            Action::OpenTaskExternal(raw_id) => {
                let id = match self.resolve_id(&raw_id) {
                    Ok(id) => id,
                    Err(e) => {
                        self.set_status_message(format!("{}", e));
                        return Ok(());
                    }
                };
                let task = match self.store.get(&id) {
                    Ok(t) => t,
                    Err(e) => {
                        self.set_status_message(format!("{}", e));
                        return Ok(());
                    }
                };
                let data_dir = self.config.resolved_data_dir()?;
                let path = data_dir.join(format!("{}.md", task.id));
                match self.open_in_new_tab(&path) {
                    Ok(true) => {
                        self.watch_file(path, WatchedFileAction::ReloadTasks);
                        self.set_status_message(format!("Task '{}' opened in editor tab", task.title));
                    }
                    Ok(false) => {
                        self.refresh_tasks()?;
                        self.set_status_message(format!("Task '{}' updated", task.title));
                    }
                    Err(e) => {
                        self.set_status_message(format!("Editor error: {}", e));
                    }
                }
            }
            Action::EditTask { id: raw_id, fields } => {
                let id = match self.resolve_id(&raw_id) {
                    Ok(id) => id,
                    Err(e) => {
                        self.set_status_message(format!("{}", e));
                        return Ok(());
                    }
                };
                let mut updated = false;
                for (field_name, value) in &fields {
                    let task_field = match TaskField::from_str(field_name) {
                        Some(f) => f,
                        None => {
                            self.set_status_message(format!(
                                "Unknown field '{}'. Valid: title, description, due, status",
                                field_name
                            ));
                            return Ok(());
                        }
                    };
                    match self.store.update(&id, task_field, value) {
                        Ok(_) => updated = true,
                        Err(e) => {
                            self.set_status_message(format!("{}", e));
                            return Ok(());
                        }
                    }
                }
                if updated {
                    self.refresh_tasks()?;
                    self.set_status_message("Task updated".into());
                }
            }
            Action::DeleteTask(id) => {
                match self.store.delete(&id) {
                    Ok(_) => {
                        self.refresh_tasks()?;
                        self.set_status_message("Task deleted".into());
                    }
                    Err(e) => {
                        self.set_status_message(format!("{}", e));
                    }
                }
            }
            Action::ShowConfirmDelete(raw_id) => {
                let id = match self.resolve_id(&raw_id) {
                    Ok(id) => id,
                    Err(e) => {
                        self.set_status_message(format!("{}", e));
                        return Ok(());
                    }
                };
                let task = match self.store.get(&id) {
                    Ok(t) => t,
                    Err(e) => {
                        self.set_status_message(format!("{}", e));
                        return Ok(());
                    }
                };
                self.mode = AppMode::Modal;
                self.modal.show_confirm_delete(id, &task.title);
            }
            Action::ToggleDoneSelected => {
                let idx = self.task_list.selected_index;
                if let Some(task) = self.tasks.get(idx) {
                    let id = task.id.clone();
                    let new_status = if task.status == Status::Done {
                        "open"
                    } else {
                        "done"
                    };
                    if self.store.update(&id, TaskField::Status, new_status).is_ok() {
                        self.refresh_tasks()?;
                    }
                }
            }
            Action::ShowDetailSelected => {
                let idx = self.task_list.selected_index;
                if idx < self.tasks.len() {
                    self.mode = AppMode::Modal;
                    self.modal.show_detail(idx);
                }
            }
            Action::DeleteSelected => {
                let idx = self.task_list.selected_index;
                if let Some(task) = self.tasks.get(idx) {
                    self.mode = AppMode::Modal;
                    self.modal.show_confirm_delete(task.id.clone(), &task.title);
                }
            }
            Action::CloneTask(raw_id) => {
                let id = match self.resolve_id(&raw_id) {
                    Ok(id) => id,
                    Err(e) => {
                        self.set_status_message(format!("{}", e));
                        return Ok(());
                    }
                };
                match self.store.clone_task(&id) {
                    Ok(cloned) => {
                        self.refresh_tasks()?;
                        self.set_status_message(format!("Task cloned as '{}' (id: {})", cloned.title, cloned.id));
                    }
                    Err(e) => {
                        self.set_status_message(format!("{}", e));
                    }
                }
            }
            Action::SetFilter(filter) => {
                self.filter = filter;
                if let Err(e) = self.refresh_tasks() {
                    self.set_status_message(format!("{}", e));
                }
            }
            Action::ShowDetail(index) => {
                if index < self.tasks.len() {
                    self.mode = AppMode::Modal;
                    self.modal.show_detail(index);
                }
            }
            Action::CloseModal => {
                self.modal.close();
                self.mode = AppMode::Normal;
            }
            Action::ToggleCommandBar => {
                if self.command_bar.visible {
                    self.command_bar.hide();
                    self.mode = AppMode::Normal;
                } else {
                    self.command_bar.show();
                    self.mode = AppMode::Command;
                }
            }
            Action::OpenConfig => {
                let config_path: PathBuf = match Config::config_path() {
                    Ok(p) => p,
                    Err(e) => {
                        self.set_status_message(format!("{}", e));
                        return Ok(());
                    }
                };
                match self.open_in_new_tab(&config_path) {
                    Ok(true) => {
                        self.watch_file(config_path, WatchedFileAction::ReloadConfig);
                        self.set_status_message("Config opened in editor tab".into());
                    }
                    Ok(false) => {
                        self.config = match Config::load() {
                            Ok(c) => c,
                            Err(e) => {
                                self.set_status_message(format!("Config reload error: {}", e));
                                return Ok(());
                            }
                        };
                        self.set_status_message("Config updated".into());
                    }
                    Err(e) => {
                        self.set_status_message(format!("Editor error: {}", e));
                    }
                }
            }
            Action::ConfigAction { key, value } => {
                match (key.as_deref(), value.as_deref()) {
                    (None, None) => {
                        self.handle_action(Action::OpenConfig)?;
                    }
                    (Some(k), None) => match self.config.get(k) {
                        Some(v) => self.set_status_message(format!("{} = {}", k, v)),
                        None => self.set_status_message(format!("Unknown key: {}", k)),
                    },
                    (Some(k), Some(v)) => match self.config.set(k, v) {
                        Ok(_) => self.set_status_message(format!("{} set to {}", k, v)),
                        Err(e) => self.set_status_message(format!("{}", e)),
                    },
                    (None, Some(_)) => {
                        self.set_status_message("Cannot set a value without a key".into());
                    }
                }
            }
            Action::MigrateData(path) => {
                let resolved = if path.starts_with('~') {
                    match dirs::home_dir() {
                        Some(home) => home.join(&path[2..]),
                        None => {
                            self.set_status_message("Could not resolve home directory".into());
                            return Ok(());
                        }
                    }
                } else {
                    PathBuf::from(&path)
                };
                let target = resolved.join("easytodo");
                match self.store.migrate(&target) {
                    Ok(_) => {
                        let new_path_str = target.to_string_lossy().to_string();
                        self.config.data_dir = if let Some(home) = dirs::home_dir() {
                            if let Ok(rel) = resolved.strip_prefix(&home) {
                                format!("~/{}", rel.to_string_lossy())
                            } else {
                                new_path_str
                            }
                        } else {
                            new_path_str
                        };
                        if let Err(e) = self.config.save() {
                            self.set_status_message(format!("Config save error: {}", e));
                        } else {
                            if let Err(e) = self.refresh_tasks() {
                                self.set_status_message(format!("Data migrated but reload failed: {}", e));
                            } else {
                                self.set_status_message("Data migrated".into());
                            }
                        }
                    }
                    Err(e) => {
                        self.set_status_message(format!("Migration error: {}", e));
                    }
                }
            }
            Action::Reload => {
                self.config = Config::load().unwrap_or_else(|_| self.config.clone());
                if let Err(e) = self.refresh_tasks() {
                    self.set_status_message(format!("Reload error: {}", e));
                } else {
                    self.set_status_message("Reloaded".into());
                }
            }
            Action::SetStatusMessage(msg) => {
                self.set_status_message(msg);
            }
            Action::Quit => {
                self.should_quit = true;
            }
        }
        Ok(())
    }

    fn detect_terminal() -> Option<(String, Vec<String>)> {
        let pid = std::process::id() as u64;

        let ppid = std::fs::read_to_string(format!("/proc/{}/status", pid))
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("PPid:"))
                    .and_then(|l| l.split_whitespace().nth(1))
                    .and_then(|p| p.parse::<u64>().ok())
            });

        let term_pid = ppid.and_then(|ppid| {
            std::fs::read_to_string(format!("/proc/{}/status", ppid))
                .ok()
                .and_then(|s| {
                    s.lines()
                        .find(|l| l.starts_with("PPid:"))
                        .and_then(|l| l.split_whitespace().nth(1))
                        .and_then(|p| p.parse::<u64>().ok())
                })
        });

        let term_name = term_pid.and_then(|pid| {
            std::fs::read_to_string(format!("/proc/{}/comm", pid))
                .ok()
                .map(|s| s.trim().to_string())
        });

        match term_name.as_deref() {
            Some("gnome-terminal-server") => {
                Some(("gnome-terminal".into(), vec!["--tab".into(), "--".into()]))
            }
            Some("konsole") => Some(("konsole".into(), vec!["--new-tab".into(), "-e".into()])),
            Some("xfce4-terminal") => {
                Some(("xfce4-terminal".into(), vec!["--tab".into(), "-e".into()]))
            }
            Some("kitty") => {
                Some(("kitty".into(), vec!["@".into(), "launch".into(), "--type".into(), "tab".into()]))
            }
            Some("alacritty") => {
                Some(("alacritty".into(), vec!["-e".into()]))
            }
            Some("wezterm") => {
                Some(("wezterm".into(), vec!["cli".into(), "spawn".into()]))
            }
            Some("foot") => {
                Some(("foot".into(), vec!["-e".into()]))
            }
            _ => None,
        }
    }

    fn open_in_new_tab(&self, path: &Path) -> Result<bool> {
        let editor = self.config.editor();

        if std::env::var("TMUX").is_ok() {
            let status = std::process::Command::new("tmux")
                .args(["new-window", &editor, &path.to_string_lossy()])
                .status();
            if let Ok(s) = status {
                if s.success() {
                    return Ok(true);
                }
            }
        }

        if let Some((term, base_args)) = Self::detect_terminal() {
            let mut args = base_args.clone();
            args.push(editor.clone());
            args.push(path.to_string_lossy().to_string());

            let status = std::process::Command::new(&term)
                .args(&args)
                .status();
            if let Ok(s) = status {
                if s.success() {
                    return Ok(true);
                }
            }
        }

        let mut stdout = io::stdout();
        execute!(stdout, LeaveAlternateScreen).map_err(|e| AppError::Terminal(e.to_string()))?;
        disable_raw_mode()?;

        let status = std::process::Command::new(&editor)
            .arg(path)
            .status()
            .map_err(|e| AppError::Editor(format!("Failed to run editor '{}': {}", editor, e)))?;

        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen).map_err(|e| AppError::Terminal(e.to_string()))?;

        if !status.success() {
            return Err(AppError::Editor(format!(
                "Editor '{}' exited with status: {}",
                editor, status
            )));
        }

        Ok(false)
    }

    fn watch_file(&mut self, path: PathBuf, action: WatchedFileAction) {
        let last_modified = std::fs::metadata(&path)
            .ok()
            .and_then(|m| m.modified().ok())
            .unwrap_or_else(|| SystemTime::now());
        self.watched_files.push(WatchedFile { path, last_modified, action });
    }

    fn check_watched_files(&mut self) {
        let mut reloaded = Vec::new();
        for (i, wf) in self.watched_files.iter().enumerate() {
            let current = std::fs::metadata(&wf.path)
                .ok()
                .and_then(|m| m.modified().ok());
            if let Some(current) = current {
                if current > wf.last_modified {
                    reloaded.push(i);
                }
            }
        }
        for i in reloaded.into_iter().rev() {
            let wf = self.watched_files.swap_remove(i);
            match wf.action {
                WatchedFileAction::ReloadConfig => {
                    self.config = Config::load().unwrap_or_else(|_| self.config.clone());
                    self.set_status_message("Config reloaded".into());
                }
                WatchedFileAction::ReloadTasks => {
                    let _ = self.refresh_tasks();
                    self.set_status_message("Task file changed, list refreshed".into());
                }
            }
        }
    }
}

fn handle_events(app: &mut App) -> Result<()> {
    if event::poll(Duration::from_millis(100))? {
        let event = event::read()?;
        let gap = app.last_event_time.elapsed();
        app.last_event_time = Instant::now();
        if gap > Duration::from_millis(500) {
            app.needs_clear = true;
        }
        if let Event::Key(key) = event {
            let is_ctrl_p = key.code == KeyCode::Char('p')
                && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL);
            let is_ctrl_n = key.code == KeyCode::Char('n')
                && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL);
            let is_ctrl_e = key.code == KeyCode::Char('e')
                && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL);
            let is_ctrl_q = key.code == KeyCode::Char('q')
                && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL);
            let is_ctrl_b = key.code == KeyCode::Char('b')
                && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL);

            let action = match app.mode {
                AppMode::Normal => app.task_list.handle_input(&key),
                AppMode::Command => {
                    if is_ctrl_p {
                        app.command_bar.hide();
                        app.mode = AppMode::Normal;
                        None
                    } else if is_ctrl_e {
                        app.command_bar.hide();
                        Some(Action::EditTaskPrefill)
                    } else if is_ctrl_n {
                        app.command_bar.hide();
                        Some(Action::NewTaskPrefill)
                    } else if is_ctrl_b {
                        Some(Action::OpenConfig)
                    } else if is_ctrl_q {
                        Some(Action::Quit)
                    } else {
                        let was_visible = app.command_bar.visible;
                        let action = app.command_bar.handle_input(&key);
                        if was_visible && !app.command_bar.visible {
                            app.mode = AppMode::Normal;
                        }
                        action
                    }
                }
                AppMode::Modal => {
                    let was_open = app.modal.is_open();
                    let action = app.modal.handle_input(&key);
                    if was_open && !app.modal.is_open() {
                        app.mode = AppMode::Normal;
                    }
                    action
                }
            };

            if let Some(action) = action {
                app.handle_action(action)?;
            }
        }
    }
    Ok(())
}

pub fn render(frame: &mut ratatui::Frame, app: &App) {
    let [list_area, bottom_area] =
        Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(frame.area());

    app.task_list.render(frame, list_area, app);
    app.command_bar.render(frame, bottom_area, app);

    if app.mode == AppMode::Modal && app.modal.is_open() {
        app.modal.render(frame, frame.area(), app);
    }
}

pub fn run(mut app: App, terminal: &mut Terminal<impl Backend>) -> Result<()> {
    app.refresh_tasks()?;

    while !app.should_quit {
        app.clear_expired_message();
        app.check_watched_files();
        if app.needs_clear {
            terminal.clear()?;
            app.needs_clear = false;
        }
        terminal.draw(|f| render(f, &app))?;
        handle_events(&mut app)?;
    }

    Ok(())
}
