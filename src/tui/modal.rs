use std::cell::Cell;

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use easytodo::action::Action;
use crate::app::App;
use crate::tui::Component;
use crossterm::event::{KeyCode, KeyEvent};

use easytodo::task::model::Status;

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    Delete(String),
}

#[derive(Debug, Clone)]
pub enum ModalContent {
    TaskDetail(usize),
    Confirm {
        message: String,
        action: ConfirmAction,
    },
    Help,
}

pub struct ModalState {
    pub modal: Option<ModalContent>,
    help_scroll: Cell<usize>,
    help_max_scroll: Cell<usize>,
}

impl ModalState {
    pub fn new() -> Self {
        ModalState { modal: None, help_scroll: Cell::new(0), help_max_scroll: Cell::new(0) }
    }

    pub fn show_detail(&mut self, index: usize) {
        self.modal = Some(ModalContent::TaskDetail(index));
    }

    pub fn show_confirm_delete(&mut self, id: String, title: &str) {
        self.modal = Some(ModalContent::Confirm {
            message: format!("Delete '{}'?", title),
            action: ConfirmAction::Delete(id),
        });
    }

    pub fn show_help(&mut self) {
        self.help_scroll.set(0);
        self.help_max_scroll.set(0);
        self.modal = Some(ModalContent::Help);
    }

    pub fn close(&mut self) {
        self.modal = None;
    }

    pub fn is_open(&self) -> bool {
        self.modal.is_some()
    }
}

impl Component for ModalState {
    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        let Some(ref content) = self.modal else {
            return;
        };

        let modal_width = area.width.min(60).max(40);
        let modal_height = match content {
            ModalContent::TaskDetail(_) => area.height.min(16).max(10),
            ModalContent::Confirm { .. } => 5,
            ModalContent::Help => area.height.min(22).max(18),
        };

        let vert_padding = (area.height.saturating_sub(modal_height)) / 2;
        let horiz_padding = (area.width.saturating_sub(modal_width)) / 2;

        let modal_area = Rect {
            x: area.x + horiz_padding,
            y: area.y + vert_padding,
            width: modal_width,
            height: modal_height,
        };

        frame.render_widget(Clear, modal_area);

        match content {
            ModalContent::TaskDetail(index) => {
                self.render_task_detail(frame, modal_area, app, *index);
            }
            ModalContent::Confirm { message, .. } => {
                self.render_confirm(frame, modal_area, message);
            }
            ModalContent::Help => {
                self.render_help(frame, modal_area, app);
            }
        }
    }

    fn handle_input(&mut self, event: &KeyEvent) -> Option<Action> {
        if self.modal.is_none() {
            return None;
        }

        match self.modal.as_ref().unwrap() {
            ModalContent::TaskDetail(_) => match event.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.close();
                    Some(Action::CloseModal)
                }
                _ => None,
            },
            ModalContent::Help => match event.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.close();
                    Some(Action::CloseModal)
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let cur = self.help_scroll.get();
                    if cur > 0 {
                        self.help_scroll.set(cur - 1);
                    }
                    None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let cur = self.help_scroll.get();
                    let max = self.help_max_scroll.get();
                    if cur < max {
                        self.help_scroll.set(cur + 1);
                    }
                    None
                }
                _ => None,
            },
            ModalContent::Confirm { action, .. } => match event.code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                    let action = action.clone();
                    self.close();
                    match action {
                        ConfirmAction::Delete(id) => Some(Action::DeleteTask(id)),
                    }
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc | KeyCode::Char('q') => {
                    self.close();
                    Some(Action::CloseModal)
                }
                _ => None,
            },
        }
    }
}

impl ModalState {
    fn render_task_detail(&self, frame: &mut Frame, area: Rect, app: &App, index: usize) {
        let Some(task) = app.tasks.get(index) else {
            return;
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.border))
            .title(" Task Detail ")
            .title_style(Style::default().fg(app.theme.title_fg));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let due = task.due_date.map(|d| d.to_string()).unwrap_or_else(|| "-".into());
        let status_str = match task.status {
            Status::Open => "Open",
            Status::Done => "Done",
        };

        let lines = vec![
            Line::from(Span::raw("")),
            Line::from(vec![
                Span::styled("Title:   ", Style::default().fg(app.theme.status_bar_fg)),
                Span::styled(&task.title, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Status:  ", Style::default().fg(app.theme.status_bar_fg)),
                Span::styled(status_str, Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("Due:     ", Style::default().fg(app.theme.status_bar_fg)),
                Span::styled(due, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Created: ", Style::default().fg(app.theme.status_bar_fg)),
                Span::styled(
                    task.created_at.format("%Y-%m-%d %H:%M").to_string(),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled("Updated: ", Style::default().fg(app.theme.status_bar_fg)),
                Span::styled(
                    task.updated_at.format("%Y-%m-%d %H:%M").to_string(),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(Span::raw("")),
            Line::from(Span::styled(
                "Description:",
                Style::default().fg(app.theme.status_bar_fg),
            )),
            Line::from(Span::raw(&task.description)),
            Line::from(Span::raw("")),
            Line::from(Span::styled(
                "  [q/Esc to close]  ",
                Style::default().fg(app.theme.status_bar_fg),
            )),
        ];

        let text = Text::from(lines);
        frame.render_widget(
            Paragraph::new(text)
                .style(Style::default().bg(app.theme.modal_bg))
                .wrap(Wrap { trim: false }),
            inner,
        );
    }

    fn render_confirm(&self, frame: &mut Frame, area: Rect, message: &str) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red));

        let inner = block.inner(area);
        frame.render_widget(Clear, area);
        frame.render_widget(block, area);

        let text = Text::from(vec![
            Line::from(Span::raw("")),
            Line::from(Span::styled(message, Style::default().fg(Color::White))),
            Line::from(Span::raw("")),
            Line::from(Span::styled(
                "  [y]es / [n]o  ",
                Style::default().fg(Color::DarkGray),
            )),
        ]);

        frame.render_widget(
            Paragraph::new(text).style(Style::default().bg(Color::Black)),
            inner,
        );
    }

    fn render_help(&self, frame: &mut Frame, area: Rect, app: &App) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.border))
            .title(" Help ")
            .title_style(Style::default().fg(app.theme.title_fg));

        let inner = block.inner(area);
        frame.render_widget(Clear, area);
        frame.render_widget(block, area);

        let accent = Style::default().fg(app.theme.status_bar_fg);
        let bright = Style::default().fg(Color::White);
        let desc = Style::default().fg(Color::Rgb(150, 155, 170));

        let all_lines = vec![
            Line::from(vec![Span::styled("Commands (Ctrl+P):", accent)]),
            Line::from(Span::raw("")),
            Line::from(vec![
                Span::styled("  new ", bright),
                Span::styled("\"<title>\" [\"<desc>\"] [due:YYYY-MM-DD]", bright),
            ]),
            Line::from(Span::styled("    Create a task", desc)),
            Line::from(Span::raw("")),
            Line::from(vec![
                Span::styled("  edit <id> ", bright),
                Span::styled("[title:\"...\"] [desc:\"...\"] [due:...] [status:...]", bright),
            ]),
            Line::from(Span::styled("    Edit task fields", desc)),
            Line::from(Span::raw("")),
            Line::from(Span::styled("  done|undone <id>", bright)),
            Line::from(Span::styled("    Mark done/not done", desc)),
            Line::from(Span::raw("")),
            Line::from(Span::styled("  delete|clone|open <id>", bright)),
            Line::from(Span::styled("    Delete, duplicate, or edit task", desc)),
            Line::from(Span::raw("")),
            Line::from(Span::styled("  list [all|todo|done]", bright)),
            Line::from(Span::styled("    List tasks with filter", desc)),
            Line::from(Span::raw("")),
            Line::from(Span::styled("  config [key] [value]", bright)),
            Line::from(Span::styled("    View or set config", desc)),
            Line::from(Span::raw("")),
            Line::from(Span::styled("  migrate <path>", bright)),
            Line::from(Span::styled("    Move task storage", desc)),
            Line::from(Span::raw("")),
            Line::from(Span::styled("  reload | help | quit", bright)),
            Line::from(Span::styled("    Reload, show help, or exit", desc)),
            Line::from(Span::raw("")),
            Line::from(vec![
                Span::styled("  Aliases: ", accent),
                Span::styled("rm=delete  cp=clone  ls=list  mv=migrate  q=quit  refresh=reload", desc),
            ]),
            Line::from(Span::styled("  \".\" resolves to the selected task", desc)),
            Line::from(Span::raw("")),
            Line::from(vec![Span::styled("Shortcuts:", accent)]),
            Line::from(vec![
                Span::styled("  j/k/↑↓        ", bright),
                Span::styled("Navigate", desc),
            ]),
            Line::from(vec![
                Span::styled("  Enter/Space   ", bright),
                Span::styled("Toggle done", desc),
            ]),
            Line::from(vec![
                Span::styled("  l/o            ", bright),
                Span::styled("Open detail", desc),
            ]),
            Line::from(vec![
                Span::styled("  1/2/3         ", bright),
                Span::styled("Filter all/todo/done", desc),
            ]),
            Line::from(vec![
                Span::styled("  Ctrl+N        ", bright),
                Span::styled("New task", desc),
            ]),
            Line::from(vec![
                Span::styled("  Ctrl+E        ", bright),
                Span::styled("Edit task", desc),
            ]),
            Line::from(vec![
                Span::styled("  Ctrl+D        ", bright),
                Span::styled("Delete task", desc),
            ]),
            Line::from(vec![
                Span::styled("  Ctrl+B        ", bright),
                Span::styled("Open config", desc),
            ]),
            Line::from(vec![
                Span::styled("  Ctrl+H        ", bright),
                Span::styled("Show help", desc),
            ]),
            Line::from(vec![
                Span::styled("  Ctrl+P        ", bright),
                Span::styled("Command bar", desc),
            ]),
            Line::from(vec![
                Span::styled("  Ctrl+Q        ", bright),
                Span::styled("Quit", desc),
            ]),
            Line::from(vec![
                Span::styled("  Esc           ", bright),
                Span::styled("Close/back", desc),
            ]),
            Line::from(Span::raw("")),
            Line::from(Span::styled("  Theme & keys: edit ~/.config/easytodo/config.toml", desc)),
            Line::from(Span::raw("")),
            Line::from(Span::styled("  [q/Esc to close]  ", desc)),
        ];

        let max = all_lines.len().saturating_sub(1);
        self.help_max_scroll.set(max);
        let scroll = self.help_scroll.get().min(max);
        self.help_scroll.set(scroll);

        let lines: Vec<Line> = all_lines.iter().skip(scroll).cloned().collect();
        let text = Text::from(lines);

        let paragraph = Paragraph::new(text)
            .style(Style::default().bg(app.theme.modal_bg))
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, inner);
    }
}
