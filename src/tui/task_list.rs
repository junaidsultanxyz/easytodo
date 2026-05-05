use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use easytodo::action::Action;
use easytodo::config::KeybindingsConfig;
use crate::app::App;
use crate::tui::Component;

use easytodo::task::model::Filter;
use easytodo::task::model::Status;

pub struct TaskList {
    pub selected_index: usize,
    pub scroll_offset: usize,
    bindings: KeybindingsConfig,
}

impl TaskList {
    pub fn new(bindings: KeybindingsConfig) -> Self {
        TaskList {
            selected_index: 0,
            scroll_offset: 0,
            bindings,
        }
    }

    pub fn update_bindings(&mut self, bindings: KeybindingsConfig) {
        self.bindings = bindings;
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

    fn binding(&self, event: &KeyEvent, key: &str) -> bool {
        if let Some(ch) = key.strip_prefix("Ctrl+") {
            if ch.len() == 1 {
                let c = ch.chars().next().unwrap().to_ascii_lowercase();
                return event.code == KeyCode::Char(c)
                    && event.modifiers.contains(KeyModifiers::CONTROL);
            }
            return false;
        }
        match key {
            "Enter" => event.code == KeyCode::Enter,
            "Space" => event.code == KeyCode::Char(' '),
            "Esc" | "Escape" => event.code == KeyCode::Esc,
            "Up" | "UpArrow" => event.code == KeyCode::Up,
            "Down" | "DownArrow" => event.code == KeyCode::Down,
            "Left" | "LeftArrow" => event.code == KeyCode::Left,
            "Right" | "RightArrow" => event.code == KeyCode::Right,
            s if s.len() == 1 => {
                event.code == KeyCode::Char(s.chars().next().unwrap())
            }
            _ => false,
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
        if self.binding(event, &self.bindings.move_down) || matches!(event.code, KeyCode::Down) {
            return Some(Action::MoveDown);
        }
        if self.binding(event, &self.bindings.move_up) || matches!(event.code, KeyCode::Up) {
            return Some(Action::MoveUp);
        }
        if self.binding(event, &self.bindings.toggle_done) || event.code == KeyCode::Char(' ') {
            return Some(Action::ToggleDoneSelected);
        }
        if self.binding(event, &self.bindings.show_detail) || event.code == KeyCode::Char('o') {
            return Some(Action::ShowDetailSelected);
        }
        if self.binding(event, &self.bindings.filter_all) {
            return Some(Action::SetFilter(Filter::All));
        }
        if self.binding(event, &self.bindings.filter_todo) {
            return Some(Action::SetFilter(Filter::Todo));
        }
        if self.binding(event, &self.bindings.filter_done) {
            return Some(Action::SetFilter(Filter::Done));
        }
        if self.binding(event, &self.bindings.new_task) {
            return Some(Action::NewTaskPrefill);
        }
        if self.binding(event, &self.bindings.edit_task) {
            return Some(Action::EditTaskPrefill);
        }
        if self.binding(event, &self.bindings.delete_task) {
            return Some(Action::DeleteSelected);
        }
        if self.binding(event, &self.bindings.open_config) {
            return Some(Action::OpenConfig);
        }
        if self.binding(event, &self.bindings.command_bar) {
            return Some(Action::ToggleCommandBar);
        }
        if self.binding(event, &self.bindings.help) {
            return Some(Action::HelpCommand);
        }
        if self.binding(event, &self.bindings.reload) {
            return Some(Action::Reload);
        }
        if self.binding(event, &self.bindings.quit) {
            return Some(Action::Quit);
        }
        None
    }
}
