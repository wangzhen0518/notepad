use std::{cmp, iter};

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
    highlighted: bool,
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
            highlighted: false,
        }
    }
}

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

    #[inline]
    pub fn set_highlighted(&mut self) {
        self.highlighted = true;
    }

    #[inline]
    pub fn reset_highlighted(&mut self) {
        self.highlighted = false;
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
                highlighted: false,
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

    pub fn highlight(
        &mut self,
        opts: &HighlightingOptions,
        word: Option<&String>,
        start_with_comment: bool,
    ) -> bool {
        if self.highlighted && word.is_none() {
            if let Some(hl_type) = self.highlighting.last() {
                return *hl_type == HighlightType::MultilineComment
                    && self.content.len() > 1
                    && &self.content[self.content.len().saturating_sub(2)..] == "*/";
            }
        }

        self.highlighting.clear();
        let chars: Vec<char> = self.content.chars().collect();
        let mut index = 0;
        let mut in_ml_comment = start_with_comment;
        if in_ml_comment {
            let closing_index = self
                .content
                .find("*/")
                .map_or(chars.len(), |cl_idx| cl_idx + 2);
            self.highlighting.append(
                &mut iter::repeat(HighlightType::MultilineComment)
                    .take(closing_index)
                    .collect(),
            );
            index = closing_index;
        }

        while let Some(c) = chars.get(index) {
            if self.highlight_multiline_comment(&mut index, opts, *c, &chars) {
                in_ml_comment = true;
                continue;
            }
            in_ml_comment = false;
            if self.highlight_char(&mut index, opts, *c, &chars)
                || self.highlight_comment(&mut index, opts, *c, &chars)
                || self.highlight_primary_keywords(&mut index, opts, &chars)
                || self.highlight_secondary_keywords(&mut index, opts, &chars)
                || self.highlight_string(&mut index, opts, *c, &chars)
                || self.highlight_number(&mut index, opts, *c, &chars)
            {
                continue;
            }
            self.highlighting.push(HighlightType::None);
            index += 1;
        }
        self.highlight_match(word);
        self.highlighted = true;

        in_ml_comment && &self.content[self.content.len().saturating_sub(2)..] != "*/"
    }
}

impl Row {
    fn highlight_match(&mut self, word: Option<&String>) {
        if let Some(word) = word {
            if word.is_empty() {
                return;
            }

            let mut index = 0;
            let word_len = word.graphemes(true).count();
            while let Some(search_match) = self.find(word, index, SearchDirection::Forward) {
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
        opts: &HighlightingOptions,
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

    fn highlight_multiline_comment(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        c: char,
        chars: &[char],
    ) -> bool {
        if opts.comments() && c == '/' && *index < chars.len() {
            if let Some(next_char) = chars.get(index.saturating_add(1)) {
                if *next_char == '*' {
                    let closing_index =
                        if let Some(closing_index) = self.content[*index + 2..].find("*/") {
                            *index + closing_index + 4
                        } else {
                            chars.len()
                        };
                    for _ in *index..closing_index {
                        self.highlighting.push(HighlightType::MultilineComment);
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
        opts: &HighlightingOptions,
        c: char,
        chars: &[char],
    ) -> bool {
        if opts.strings() && c == '"' {
            *index += 1;
            while let Some(next_char) = chars.get(*index) {
                if *next_char != '"' {
                    self.highlighting.push(HighlightType::String);
                    *index += 1;
                } else {
                    break;
                }
            }
            self.highlighting.push(HighlightType::String); // current char
            self.highlighting.push(HighlightType::String); // next char: '"'
            *index += 1;
            return true;
        }
        false
    }

    fn highlight_number(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        c: char,
        chars: &[char],
    ) -> bool {
        if opts.numbers() && c.is_ascii_digit() {
            if *index != 0 {
                let prev_char: char = chars[*index - 1];
                if !is_separator(prev_char) {
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
            return true;
        }
        false
    }

    fn hightlight_str(
        &mut self,
        index: &mut usize,
        substring: &str,
        chars: &[char],
        hl_type: &HighlightType,
    ) -> bool {
        if substring.is_empty() {
            return false;
        }

        for (substr_index, c) in substring.chars().enumerate() {
            if let Some(next_char) = chars.get(*index + substr_index) {
                if *next_char != c {
                    return false;
                }
            } else {
                return false;
            }
        }
        self.highlighting.append(
            &mut iter::repeat(hl_type.clone())
                .take(substring.len())
                .collect(),
        );
        *index += substring.len();
        true
    }

    fn highlight_keywords(
        &mut self,
        index: &mut usize,
        chars: &[char],
        keywords: &[&'static str],
        hl_type: &HighlightType,
    ) -> bool {
        if *index > 0 {
            let prev_char = chars[*index - 1];
            if !is_separator(prev_char) {
                return false;
            }
        }
        for word in keywords.iter() {
            let next_index = *index + word.len();
            if let Some(&next_char) = chars.get(next_index) {
                if !is_separator(next_char) {
                    continue;
                }
            }
            if self.hightlight_str(index, word, chars, hl_type) {
                return true;
            }
        }
        false
    }

    fn highlight_primary_keywords(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        chars: &[char],
    ) -> bool {
        self.highlight_keywords(
            index,
            chars,
            opts.primary_keywords(),
            &HighlightType::PrimaryKeywords,
        )
    }

    fn highlight_secondary_keywords(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        chars: &[char],
    ) -> bool {
        self.highlight_keywords(
            index,
            chars,
            opts.secondary_keywords(),
            &HighlightType::SecondaryKeywords,
        )
    }
}

fn is_separator(c: char) -> bool {
    c.is_ascii_punctuation() || c.is_whitespace()
}
