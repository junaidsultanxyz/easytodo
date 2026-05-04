use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use easytodo::action::Action;
use crate::app::App;
use crate::tui::Component;
use easytodo::commands::parser;
use crossterm::event::{KeyCode, KeyEvent};

pub struct CommandBar {
    pub visible: bool,
    pub input: String,
    pub cursor_position: u16,
}

impl CommandBar {
    pub fn new() -> Self {
        CommandBar {
            visible: false,
            input: String::new(),
            cursor_position: 0,
        }
    }

    pub fn clear(&mut self) {
        self.input.clear();
        self.cursor_position = 0;
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.clear();
    }

    pub fn show_with_text(&mut self, text: &str) {
        self.visible = true;
        self.input = text.to_string();
        self.cursor_position = text.len() as u16;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.clear();
    }
}

impl Component for CommandBar {
    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        if !self.visible {
            let status_text = match &app.status_message {
                Some(msg) => msg.clone(),
                None => String::new(),
            };
            let style = Style::default().fg(app.theme.status_bar_fg).bg(app.theme.normal_bg);
            let line = Line::from(Span::styled(format!(" {}", status_text), style));
            frame.render_widget(
                Paragraph::new(line).style(Style::default().bg(app.theme.normal_bg)),
                area,
            );
            return;
        }

        let prompt = ": ";
        let display_text = format!("{}{}", prompt, self.input);

        let style = Style::default()
            .fg(Color::White)
            .bg(app.theme.command_bar_bg);

        let line = Line::from(Span::styled(display_text, style));
        frame.render_widget(
            Paragraph::new(line).style(Style::default().bg(app.theme.command_bar_bg)),
            area,
        );

        if self.visible {
            let cursor_x = area.x + prompt.len() as u16 + self.cursor_position;
            let _ = frame.set_cursor_position((cursor_x, area.y));
        }
    }

    fn handle_input(&mut self, event: &KeyEvent) -> Option<Action> {
        if !self.visible {
            return None;
        }

        match event.code {
            KeyCode::Esc => {
                self.hide();
                None
            }
            KeyCode::Enter => {
                let input = self.input.clone();
                self.hide();

                if input.trim().is_empty() {
                    return None;
                }

                match parser::parse(&input) {
                    Ok(command) => {
                        let action = match command {
                            easytodo::commands::Command::New { title, description, due } => {
                                let due_date = match &due {
                                    Some(d) if !d.is_empty() => {
                                        match chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d") {
                                            Ok(date) => Some(date),
                                            Err(_) => return Some(Action::SetStatusMessage(
                                                format!("Invalid date '{}'. Use YYYY-MM-DD", d),
                                            )),
                                        }
                                    }
                                    _ => None,
                                };
                                Some(Action::NewTask { title, description, due: due_date })
                            }
                            easytodo::commands::Command::Open(id) => {
                                Some(Action::OpenTaskExternal(id))
                            }
                            easytodo::commands::Command::Edit { id, fields } => {
                                Some(Action::EditTask { id, fields })
                            }
                            easytodo::commands::Command::Delete(id) => {
                                Some(Action::ShowConfirmDelete(id))
                            }
                            easytodo::commands::Command::Done(id) => Some(Action::EditTask {
                                id,
                                fields: vec![("status".into(), "done".into())],
                            }),
                            easytodo::commands::Command::Undone(id) => Some(Action::EditTask {
                                id,
                                fields: vec![("status".into(), "open".into())],
                            }),
                            easytodo::commands::Command::Clone(id) => Some(Action::CloneTask(id)),
                            easytodo::commands::Command::List(filter_str) => {
                                let filter = match filter_str {
                                    Some(s) => match easytodo::task::model::Filter::from_str(&s) {
                                        Some(f) => f,
                                        None => return Some(Action::SetStatusMessage(format!(
                                            "Unknown filter '{}'. Use: all, todo, done", s
                                        ))),
                                    },
                                    None => easytodo::task::model::Filter::All,
                                };
                                Some(Action::SetFilter(filter))
                            }
                            easytodo::commands::Command::Config { key, value } => {
                                Some(Action::ConfigAction { key, value })
                            }
                            easytodo::commands::Command::Migrate(path) => {
                                Some(Action::MigrateData(path))
                            }
                            easytodo::commands::Command::Reload => Some(Action::Reload),
                            easytodo::commands::Command::Quit => Some(Action::Quit),
                        };
                        action
                    }
                    Err(e) => Some(Action::SetStatusMessage(format!("Error: {}", e))),
                }
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    let pos = self.cursor_position as usize - 1;
                    self.input.remove(pos);
                    self.cursor_position -= 1;
                }
                None
            }
            KeyCode::Delete => {
                let pos = self.cursor_position as usize;
                if pos < self.input.len() {
                    self.input.remove(pos);
                }
                None
            }
            KeyCode::Left => {
                self.cursor_position = self.cursor_position.saturating_sub(1);
                None
            }
            KeyCode::Right => {
                if (self.cursor_position as usize) < self.input.len() {
                    self.cursor_position += 1;
                }
                None
            }
            KeyCode::Home => {
                self.cursor_position = 0;
                None
            }
            KeyCode::End => {
                self.cursor_position = self.input.len() as u16;
                None
            }
            KeyCode::Char(c) => {
                let pos = self.cursor_position as usize;
                self.input.insert(pos, c);
                self.cursor_position += 1;
                None
            }
            _ => None,
        }
    }
}
