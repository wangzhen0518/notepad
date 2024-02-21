// #![warn(clippy::all, clippy::pedantic, clippy::restriction)]
// #![allow(
//     clippy::missing_docs_in_private_items,
//     clippy::missing_errors_doc,
//     clippy::missing_panics_doc,
//     clippy::missing_safety_doc,
//     clippy::implicit_return,
//     clippy::shadow_reuse,
//     clippy::print_stdout,
//     clippy::wildcard_enum_match_arm,
//     clippy::else_if_without_else,
//     clippy::std_instead_of_core,
//     clippy::question_mark,
//     clippy::question_mark_used,
//     clippy::min_ident_chars,
//     clippy::inline_always,
//     clippy::missing_inline_in_public_items
// )]

use std::{cmp, collections::HashSet};

use crossterm::style::{self, Stylize};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    editor::SearchDirection,
    filetype::HighlightingOptions,
    highlighting::{self, HighlightType},
};

#[derive(Debug, Default, Clone)]
pub struct Row {
    content: String,
    highlighting: Vec<highlighting::HighlightType>,
    len: usize,
    modified: bool,
}

impl<T> From<T> for Row
where
    T: Into<String>,
{
    fn from(value: T) -> Self {
        let content: String = value.into();
        let len = content.graphemes(true).count(); // need to be change when editing
        Self {
            content,
            highlighting: Vec::new(),
            len,
            modified: false,
        }
    }
}

// impl Row {
//     fn update_len(&mut self) {
//         self.len = self.content.graphemes(true).count();
//     }
// }

impl Row {
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    #[must_use]
    #[inline]
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    #[inline]
    pub fn set_modified(&mut self) {
        self.modified = true;
    }

    #[inline]
    pub fn reset_modified(&mut self) {
        self.modified = false;
    }

    #[must_use]
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.content.as_bytes()
    }
}

impl Row {
    #[must_use]
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.content.len());
        let start = cmp::min(start, end);
        let mut result = String::new();
        let mut result_tmp = String::new();
        let mut current_highlighting = &HighlightType::None;
        #[allow(clippy::arithmetic_side_effects)]
        for (index, grapheme) in self
            .content
            .graphemes(true)
            .map(|s| if s == "\t" { " " } else { s })
            .enumerate()
            .skip(start)
            .take(end - start)
        {
            if let Some(c) = grapheme.chars().next() {
                let highlight_type = self.highlighting.get(index).unwrap_or_default();
                if highlight_type == current_highlighting {
                    result_tmp.push(c);
                } else {
                    result.push_str(&format!(
                        "{}",
                        style::style(&result_tmp).with(current_highlighting.into())
                    ));
                    result_tmp.clear();
                    result_tmp.push(c);
                    current_highlighting = highlight_type;
                }
            }
        }
        if !result_tmp.is_empty() {
            result.push_str(&format!(
                "{}",
                style::style(&result_tmp).with(current_highlighting.into())
            ));
        }
        result
    }

    #[must_use]
    pub fn split(&mut self, at: usize) -> Row {
        let mut new_row = if at >= self.len() {
            Row::default()
        } else {
            let (mut current_row, mut new_row) = (String::new(), String::new());
            let (mut current_len, mut new_len) = (0, 0);
            for (index, grapheme) in self.content.graphemes(true).enumerate() {
                if index < at {
                    current_len += 1;
                    current_row.push_str(grapheme);
                } else {
                    new_len += 1;
                    new_row.push_str(grapheme);
                }
            }
            self.content = current_row;
            self.len = current_len;
            self.set_modified();
            Row {
                content: new_row,
                highlighting: Vec::new(),
                len: new_len,
                modified: true,
            }
        };
        new_row.set_modified();
        new_row
    }

    pub fn insert(&mut self, at: usize, c: char) {
        if at >= self.len() {
            self.content.push(c);
            self.len += 1;
        } else {
            let mut result = String::new();
            let mut length = 0;
            for (index, grapheme) in self.content.graphemes(true).enumerate() {
                length += 1;
                if index == at {
                    length += 1;
                    result.push(c);
                }
                result.push_str(grapheme);
            }
            self.len = length;
            self.content = result;
        }
        self.set_modified();
    }

    pub fn delete(&mut self, at: usize) {
        if at < self.len() {
            let mut res: String = String::new();
            let mut length = 0;
            for (index, grapheme) in self.content.graphemes(true).enumerate() {
                if index != at {
                    length += 1;
                    res.push_str(grapheme);
                }
            }
            self.content = res;
            self.len = length;
            self.set_modified();
        }
    }

    pub fn append(&mut self, row: &Row) {
        self.content.push_str(&row.content);
        self.len += row.len;
        self.set_modified();
    }

    pub fn find(&self, query: &str, at: usize, direction: SearchDirection) -> Option<usize> {
        if at > self.len() || query.is_empty() {
            return None;
        }

        let (start, end) = match direction {
            SearchDirection::Forward => (at, self.len()),
            SearchDirection::Backward => (0, at),
        };
        #[allow(clippy::arithmetic_side_effects)]
        let substring: String = self
            .content
            .graphemes(true)
            .skip(start)
            .take(end - start)
            .collect();
        let matching_byte_index = match direction {
            SearchDirection::Forward => substring.find(query),
            SearchDirection::Backward => substring.rfind(query),
        };
        if let Some(matching_byte_index) = matching_byte_index {
            for (grapheme_index, (byte_index, _)) in substring.grapheme_indices(true).enumerate() {
                if matching_byte_index == byte_index {
                    #[allow(clippy::arithmetic_side_effects)]
                    return Some(start + grapheme_index);
                }
            }
        }
        None
    }

    pub fn highlight(&mut self, opts: &HighlightingOptions, word: Option<&str>) {
        let mut search_index = 0;
        let mut matches = HashSet::new();
        if let Some(word) = word {
            let word_len = word.graphemes(true).count();
            while let Some(search_match) = self.find(word, search_index, SearchDirection::Forward) {
                matches.insert(search_match);
                if let Some(next_index) = search_match.checked_add(word_len) {
                    search_index = next_index;
                } else {
                    break;
                }
            }
        }

        // let chars: Vec<char> = self.content.chars().collect();
        let chars: Vec<char> = self.content.chars().collect();
        let mut highlight_result = Vec::new();
        let mut prev_is_separtor = true;
        let mut in_string = false;
        let mut index = 0;
        while let Some(c) = chars.get(index) {
            if let Some(word) = word {
                if matches.contains(&index) {
                    for _ in word.graphemes(true) {
                        index += 1;
                        highlight_result.push(HighlightType::Match);
                    }
                    continue;
                }
            }

            let previous_highlight = if index > 0 {
                #[allow(clippy::arithmetic_side_effects)]
                highlight_result.get(index - 1).unwrap_or_default()
            } else {
                &HighlightType::None
            };

            if opts.characters() && !in_string && *c == '\'' {
                prev_is_separtor = true;
                if let Some(next_char) = chars.get(index.saturating_add(1)) {
                    let closing_index = if *next_char == '\\' {
                        index.saturating_add(3)
                    } else {
                        index.saturating_add(2)
                    };
                    if let Some(closing_char) = chars.get(closing_index) {
                        if *closing_char == '\'' {
                            #[allow(clippy::mut_range_bound)]
                            for _ in index..=closing_index {
                                highlight_result.push(HighlightType::Character);
                                index += 1; // range bound keep unchanged
                            }
                            continue;
                        }
                    }
                }
            }

            if opts.strings() {
                if in_string {
                    highlight_result.push(HighlightType::String);
                    if *c == '\\' && index < self.len().saturating_sub(1) {
                        highlight_result.push(HighlightType::String);
                        index += 1;
                    } else if *c == '"' {
                        in_string = false;
                        prev_is_separtor = true;
                    } else {
                        prev_is_separtor = false;
                    }
                    index += 1;
                    continue;
                } else if prev_is_separtor && *c == '"' {
                    highlight_result.push(HighlightType::String);
                    in_string = true;
                    prev_is_separtor = true;
                    index += 1;
                    continue;
                }
            }

            if opts.numbers()
                && ((c.is_ascii_digit()
                    && (prev_is_separtor || *previous_highlight == HighlightType::Number))
                    || (*c == '.' && *previous_highlight == HighlightType::Number))
            {
                highlight_result.push(HighlightType::Number);
                prev_is_separtor = false;
                index += 1;
                continue;
            }

            highlight_result.push(HighlightType::None);
            prev_is_separtor = c.is_ascii_punctuation() || c.is_ascii_whitespace();
            index += 1;
        }
        self.highlighting = highlight_result;
    }
}

impl Row {
    fn highlight_match(&mut self, word: Option<&str>) {
        if let Some(word) = word {
            if word.is_empty() {
                return;
            }

            let mut index = 0;
            // let mut matches = HashSet::new();
            let word_len = word.graphemes(true).count();
            while let Some(search_match) = self.find(word, index, SearchDirection::Forward) {
                // matches.insert(search_match);
                if let Some(next_index) = search_match.checked_add(word_len) {
                    for i in search_match..next_index {
                        self.highlighting[i] = HighlightType::Match;
                    }
                    index = next_index;
                } else {
                    break;
                }
            }
        }
    }

    fn highlight_char(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        c: char,
        chars: &[char],
    ) -> bool {
        if opts.characters() && c == '\'' {
            if let Some(next_char) = chars.get(index.saturating_add(1)) {
                let closing_index = if *next_char == '\\' {
                    index.saturating_add(3)
                } else {
                    index.saturating_add(2)
                };
                if let Some(closing_char) = chars.get(closing_index) {
                    if *closing_char == '\'' {
                        for _ in *index..=closing_index {
                            self.highlighting.push(HighlightType::Character);
                        }
                        *index = closing_index + 1;
                        return true;
                    }
                }
            }
        }
        false
    }

    fn highlight_comment(
        &mut self,
        index: &mut usize,
        opts: HighlightingOptions,
        c: char,
        chars: &[char],
    ) -> bool {
        if opts.comments() && c == '/' && *index < chars.len() {
            if let Some(next_char) = chars.get(index.saturating_add(1)) {
                if *next_char == '/' {
                    for _ in *index..chars.len() {
                        self.highlighting.push(HighlightType::Comment);
                        *index += 1;
                    }
                    return true;
                }
            };
        }
        false
    }

    fn highlight_string(
        &mut self,
        index: &mut usize,
        opts: HighlightingOptions,
        c: char,
        chars: &[char],
    ) -> bool {
        if opts.strings() && c == '"' {
            *index += 1;
            while let Some(next_char) = chars.get(*index) {
                if *next_char == '"' {
                    break;
                }
                self.highlighting.push(HighlightType::String);
                *index += 1;
            }
            self.highlighting.push(HighlightType::String);
            *index += 1;
            return true;
        }
        false
    }

    fn highlight_number(
        &mut self,
        index: &mut usize,
        opts: HighlightingOptions,
        c: char,
        chars: &[char],
    ) -> bool {
        if opts.numbers() && c.is_ascii_digit() {
            if *index != 0 {
                let prev_char: char = chars[*index - 1];
                if !(prev_char.is_ascii_punctuation() || prev_char.is_whitespace()) {
                    return false;
                }
            }
            *index += 1;
            while let Some(next_char) = chars.get(*index) {
                if next_char.is_ascii_digit() || *next_char == '.' {
                    self.highlighting.push(HighlightType::Number);
                    *index += 1;
                } else {
                    break;
                }
            }
            self.highlighting.push(HighlightType::Number);
            *index += 1;
            return true;
        }
        false
    }
}
