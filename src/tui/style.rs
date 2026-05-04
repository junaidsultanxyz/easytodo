use ratatui::style::Color;

pub struct Theme {
    pub selected_bg: Color,
    pub done_fg: Color,
    pub border: Color,
    pub command_bar_bg: Color,
    pub modal_bg: Color,
    pub title_fg: Color,
    pub normal_bg: Color,
    pub status_bar_fg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            selected_bg: Color::Rgb(60, 60, 80),
            done_fg: Color::Rgb(100, 140, 100),
            border: Color::Rgb(80, 80, 120),
            command_bar_bg: Color::Rgb(30, 30, 50),
            modal_bg: Color::Rgb(25, 25, 45),
            title_fg: Color::Rgb(180, 180, 220),
            normal_bg: Color::Rgb(20, 20, 35),
            status_bar_fg: Color::Rgb(130, 130, 160),
        }
    }
}
