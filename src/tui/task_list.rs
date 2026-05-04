use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use easytodo::action::Action;
use crate::app::App;
use crate::tui::Component;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use easytodo::task::model::Filter;
use easytodo::task::model::Status;

pub struct TaskList {
    pub selected_index: usize,
    pub scroll_offset: usize,
}

impl TaskList {
    pub fn new() -> Self {
        TaskList {
            selected_index: 0,
            scroll_offset: 0,
        }
    }

    pub fn clamp_selection(&mut self, max_len: usize) {
        if max_len == 0 {
            self.selected_index = 0;
            self.scroll_offset = 0;
            return;
        }
        if self.selected_index >= max_len {
            self.selected_index = max_len - 1;
        }
    }

}

impl Component for TaskList {
    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        let theme = &app.theme;

        let done_count = app.tasks.iter().filter(|t| t.status == Status::Done).count();
        let total_count = app.tasks.len();
        let title = format!(" EasyTodo  [{}/{}] ", done_count, total_count);

        let filter_tag = match app.filter {
            Filter::All => "all",
            Filter::Todo => "todo",
            Filter::Done => "done",
        };
        let full_title = format!("{}[{}]", title, filter_tag);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(full_title)
            .title_style(Style::default().fg(theme.title_fg))
            .title_alignment(Alignment::Left);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let visible_height = inner.height as usize;
        let mut scroll = self.scroll_offset;
        if self.selected_index < scroll {
            scroll = self.selected_index;
        } else if self.selected_index >= scroll + visible_height {
            scroll = self.selected_index - visible_height + 1;
        }

        for i in 0..visible_height {
            let task_idx = scroll + i;
            if task_idx >= app.tasks.len() {
                break;
            }

            let task = &app.tasks[task_idx];
            let row_y = inner.y + i as u16;
            let row_area = Rect::new(inner.x, row_y, inner.width, 1);

            let is_selected = task_idx == self.selected_index;

            let checkbox = if task.status == Status::Done {
                "[x]"
            } else {
                "[ ]"
            };

            let bg = if is_selected { theme.selected_bg } else { theme.normal_bg };

            let title_style = if task.status == Status::Done {
                Style::default().fg(theme.done_fg).bg(bg)
            } else {
                Style::default().fg(Color::White).bg(bg)
            };

            let checkbox_style = Style::default()
                .fg(if task.status == Status::Done {
                    Color::Green
                } else {
                    Color::DarkGray
                })
                .bg(bg);

            let date_str = task.due_date.map(|d| d.to_string()).unwrap_or_default();
            let date_width: u16 = if date_str.is_empty() { 0 } else { 12 };
            let checkbox_width: u16 = 4;

            let title_max_width = inner.width.saturating_sub(checkbox_width + date_width + 1);
            let display_title = if task.title.len() as u16 > title_max_width {
                format!(
                    "{}…",
                    &task.title[..(title_max_width.saturating_sub(1)) as usize]
                )
            } else {
                task.title.clone()
            };

            let date_style = Style::default().fg(theme.status_bar_fg).bg(bg);
            let padding_len = title_max_width.saturating_sub(display_title.len() as u16);
            let padding = " ".repeat(padding_len as usize);

            let line = Line::from(vec![
                Span::styled(format!(" {} ", checkbox), checkbox_style),
                Span::styled(display_title, title_style),
                Span::styled(padding, Style::default().bg(bg)),
                Span::styled(date_str, date_style),
            ]);

            frame.render_widget(Paragraph::new(line).style(Style::default().bg(bg)), row_area);
        }
    }

    fn handle_input(&mut self, event: &KeyEvent) -> Option<Action> {
        match event.code {
            KeyCode::Char('j') | KeyCode::Down => Some(Action::MoveDown),
            KeyCode::Char('k') | KeyCode::Up => Some(Action::MoveUp),
            KeyCode::Enter | KeyCode::Char(' ') => Some(Action::ToggleDoneSelected),
            KeyCode::Char('l') => Some(Action::ShowDetailSelected),
            KeyCode::Char('d') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Action::DeleteSelected)
            }
            KeyCode::Char('n') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Action::NewTaskPrefill)
            }
            KeyCode::Char('b') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Action::OpenConfig)
            }
            KeyCode::Char('p') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Action::ToggleCommandBar)
            }
            KeyCode::Char('e') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Action::EditTaskPrefill)
            }
            KeyCode::Char('o') => Some(Action::ShowDetailSelected),
            KeyCode::Char('q') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Action::Quit)
            }
            KeyCode::Char('1') => Some(Action::SetFilter(Filter::All)),
            KeyCode::Char('2') => Some(Action::SetFilter(Filter::Todo)),
            KeyCode::Char('3') => Some(Action::SetFilter(Filter::Done)),
            _ => None,
        }
    }
}
