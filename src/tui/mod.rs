pub mod command_bar;
pub mod modal;
pub mod style;
pub mod task_list;

use ratatui::layout::Rect;
use ratatui::Frame;

use easytodo::action::Action;
use crate::app::App;

pub trait Component {
    fn render(&self, frame: &mut Frame, area: Rect, app: &App);
    fn handle_input(&mut self, event: &KeyEvent) -> Option<Action>;
}

use crossterm::event::KeyEvent;
