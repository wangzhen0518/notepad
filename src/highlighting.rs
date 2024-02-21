use crossterm::style::Color;

use crate::constants::{
    CHARACTER_COLOR, COMMENT_COLOR, MATCH_COLOR, NONE_COLOR, NUMBER_COLOR, PRIMARY_KEYWORDS_COLOR,
    SECONDARY_KEYWORDS_COLOR, STRING_COLOR,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum HighlightType {
    #[default]
    None,
    Number,
    Match,
    String,
    Character,
    Comment,
    MultilineComment,
    PrimaryKeywords,
    SecondaryKeywords,
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
            HighlightType::None => NONE_COLOR,
            HighlightType::Number => NUMBER_COLOR,
            HighlightType::Match => MATCH_COLOR,
            HighlightType::String => STRING_COLOR,
            HighlightType::Character => CHARACTER_COLOR,
            HighlightType::Comment | HighlightType::MultilineComment => COMMENT_COLOR,
            HighlightType::PrimaryKeywords => PRIMARY_KEYWORDS_COLOR,
            HighlightType::SecondaryKeywords => SECONDARY_KEYWORDS_COLOR,
        }
    }
}

impl Default for &HighlightType {
    fn default() -> Self {
        &HighlightType::None
    }
}
