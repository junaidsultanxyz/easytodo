use ratatui::style::Color;

use easytodo::config::parse_color;

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

impl Theme {
    pub fn from_config(cfg: &easytodo::config::ThemeConfig) -> Self {
        Theme {
            selected_bg: parse_color(&cfg.selected_bg).map(|(r, g, b)| Color::Rgb(r, g, b)).unwrap_or(Color::Rgb(60, 60, 80)),
            done_fg: parse_color(&cfg.done_fg).map(|(r, g, b)| Color::Rgb(r, g, b)).unwrap_or(Color::Rgb(100, 140, 100)),
            border: parse_color(&cfg.border).map(|(r, g, b)| Color::Rgb(r, g, b)).unwrap_or(Color::Rgb(80, 80, 120)),
            command_bar_bg: parse_color(&cfg.command_bar_bg).map(|(r, g, b)| Color::Rgb(r, g, b)).unwrap_or(Color::Rgb(30, 30, 50)),
            modal_bg: parse_color(&cfg.modal_bg).map(|(r, g, b)| Color::Rgb(r, g, b)).unwrap_or(Color::Rgb(25, 25, 45)),
            title_fg: parse_color(&cfg.title_fg).map(|(r, g, b)| Color::Rgb(r, g, b)).unwrap_or(Color::Rgb(180, 180, 220)),
            normal_bg: parse_color(&cfg.normal_bg).map(|(r, g, b)| Color::Rgb(r, g, b)).unwrap_or(Color::Rgb(20, 20, 35)),
            status_bar_fg: parse_color(&cfg.status_bar_fg).map(|(r, g, b)| Color::Rgb(r, g, b)).unwrap_or(Color::Rgb(130, 130, 160)),
        }
    }
}
