use crossterm::style::Color;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum HighlightType {
    #[default]
    None,
    Number,
    Match,
    String,
    Character,
    Comment,
}

// impl From<HighlightType> for Color {
//     fn from(value: HighlightType) -> Self {
//         match value {
//             HighlightType::None => Color::White,
//             HighlightType::Number => Color::Red,
//             HighlightType::Match => Color::Yellow,
//         }
//     }
// }

impl From<&HighlightType> for Color {
    fn from(value: &HighlightType) -> Self {
        match value {
            HighlightType::None => Color::White,
            HighlightType::Number => Color::Red,
            HighlightType::Match => Color::Blue,
            HighlightType::String => Color::Rgb {
                r: 211,
                g: 54,
                b: 130,
            },
            HighlightType::Character => Color::Rgb {
                r: 108,
                g: 113,
                b: 196,
            },
            HighlightType::Comment => Color::Rgb {
                r: 133,
                g: 153,
                b: 0,
            },
        }
    }
}

impl Default for &HighlightType {
    fn default() -> Self {
        &HighlightType::None
    }
}
