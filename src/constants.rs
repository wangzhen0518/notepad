use std::env;

use crossterm::style;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const STATUS_FG_COLOR: style::Color = style::Color::Rgb {
    r: 63,
    g: 63,
    b: 63,
};
pub const STATUS_BG_COLOR: style::Color = style::Color::Rgb {
    r: 239,
    g: 239,
    b: 239,
};

pub const QUIT_TIMES: u8 = 2;
