use crossterm::style::Color;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const STATUS_FG_COLOR: Color = Color::Rgb {
    r: 63,
    g: 63,
    b: 63,
};
pub const STATUS_BG_COLOR: Color = Color::Rgb {
    r: 239,
    g: 239,
    b: 239,
};

pub const NUMBER_COLOR: Color = Color::Rgb {
    r: 220,
    g: 163,
    b: 163,
};

pub const MATCH_COLOR: Color = Color::Rgb {
    r: 38,
    g: 139,
    b: 210,
};

pub const STRING_COLOR: Color = Color::Rgb {
    r: 211,
    g: 54,
    b: 130,
};

pub const CHARACTER_COLOR: Color = Color::Rgb {
    r: 108,
    g: 113,
    b: 196,
};

pub const COMMENT_COLOR: Color = Color::Rgb {
    r: 133,
    g: 153,
    b: 0,
};

pub const PRIMARY_KEYWORDS_COLOR: Color = Color::Rgb {
    r: 181,
    g: 137,
    b: 0,
};

pub const SECONDARY_KEYWORDS_COLOR: Color = Color::Rgb {
    r: 42,
    g: 161,
    b: 152,
};

pub const NONE_COLOR: Color = Color::Rgb {
    r: 255,
    g: 255,
    b: 255,
};

pub const QUIT_TIMES: u8 = 2;
