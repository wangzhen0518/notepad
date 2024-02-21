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

use std::{
    fs,
    io::{self, Error, Write},
    ops::Index,
    path,
};

use crate::{
    editor::{Position, SearchDirection},
    filetype::FileType,
    row::Row,
};

#[derive(Debug, Default)]
pub struct Document {
    filename: Option<String>,
    rows: Vec<Row>,
    filetype: FileType,
    dirty: bool,
}

impl Document {
    #[inline]
    pub fn open(filename: &str) -> Result<Self, Error> {
        let filename = path::Path::new(filename);
        let contents = fs::read_to_string(filename)?;
        let filename = filename
            .file_name()
            .map(|s| s.to_string_lossy().into_owned());
        let filetype = FileType::from(filename.clone().unwrap_or_default());
        Ok(Self {
            filename,
            rows: contents
                .lines()
                .map(|s| {
                    let mut row = Row::from(s);
                    row.highlight(filetype.highlightling_options(), None);
                    row
                })
                .collect(),
            filetype,
            dirty: false,
        })
    }

    #[must_use]
    #[inline]
    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    #[must_use]
    #[inline]
    pub fn row_length(&self, index: usize) -> usize {
        // if let Some(row) = self.row(index) {
        //     row.len()
        // } else {
        //     0
        // }
        self.rows.get(index).map(Row::len).unwrap_or_default()
    }

    #[must_use]
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    #[must_use]
    #[inline]
    pub fn filename(&self) -> Option<&String> {
        self.filename.as_ref()
    }

    #[inline]
    pub fn set_filename(&mut self, filename: Option<String>) {
        self.filename = filename;
    }

    #[must_use]
    #[inline]
    pub fn filetype(&self) -> String {
        self.filetype.name()
    }
}

impl Document {
    pub fn new_line(&mut self, at: &Position) {
        if at.y() > self.len() {
            return;
        }

        self.dirty = true;
        if at.y() == self.len() {
            // add new line to the end of the document
            self.rows.push(Row::default());
        } else {
            // split current line to two part and create a new line

            #[allow(clippy::indexing_slicing)]
            let current_row = &mut self.rows[at.y()];
            let mut next_row = current_row.split(at.x());
            current_row.highlight(self.filetype.highlightling_options(), None);
            next_row.highlight(self.filetype.highlightling_options(), None);
            #[allow(clippy::arithmetic_side_effects)]
            self.rows.insert(at.y() + 1, next_row);
        }
    }

    pub fn insert(&mut self, at: &Position, c: char) {
        if at.y() > self.len() {
            return;
        }

        self.dirty = true;
        if at.y() == self.len() {
            let mut row = Row::default();
            row.insert(0, c);
            row.highlight(self.filetype.highlightling_options(), None);
            self.rows.push(row);
        } else {
            #[allow(clippy::indexing_slicing)]
            let row = &mut self.rows[at.y()];
            row.insert(at.x(), c);
            row.highlight(self.filetype.highlightling_options(), None);
        }
    }

    pub fn delete(&mut self, at: &Position) {
        if at.y() >= self.len() {
            return;
        }

        self.dirty = true;
        #[allow(clippy::arithmetic_side_effects)]
        if at.x() == self.row_length(at.y()) && at.y() + 1 < self.len() {
            #[allow(clippy::arithmetic_side_effects)]
            let next_row = self.rows.remove(at.y() + 1);
            #[allow(clippy::indexing_slicing)]
            let row = &mut self.rows[at.y()];
            row.append(&next_row);
            row.highlight(self.filetype.highlightling_options(), None);
        } else {
            #[allow(clippy::indexing_slicing)]
            let row = &mut self.rows[at.y()];
            row.delete(at.x());
            row.highlight(self.filetype.highlightling_options(), None);
        }
    }

    pub fn save(&mut self) -> Result<(), io::Error> {
        #[allow(clippy::pattern_type_mismatch)]
        if let Some(filename) = &self.filename {
            let mut file = fs::File::create(filename)?;
            // for row in self.rows.iter_mut().filter(|r| r.is_modified()) {
            self.filetype = FileType::from(filename);
            for row in &mut self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
                row.highlight(self.filetype.highlightling_options(), None);
            }
            self.dirty = false;
        }
        Ok(())
    }

    pub fn find(&self, query: &str, at: &Position, direction: SearchDirection) -> Option<Position> {
        if at.y() >= self.len() || query.is_empty() {
            return None;
        }

        let (start, end) = match direction {
            SearchDirection::Forward => (at.y(), self.len()),
            SearchDirection::Backward => (0, at.y().saturating_add(1)),
        };
        #[allow(clippy::arithmetic_side_effects)]
        let mut row_iter: Box<dyn DoubleEndedIterator<Item = (usize, &Row)>> =
            Box::new(self.rows.iter().enumerate().skip(start).take(end - start));
        if direction == SearchDirection::Backward {
            row_iter = Box::new(row_iter.rev());
        }
        let mut x = at.x();
        for (y, row) in row_iter {
            if let Some(x) = row.find(query, x, direction) {
                return Some(Position::new(x, y));
            }
            x = match direction {
                SearchDirection::Forward => 0,
                SearchDirection::Backward => self.row_length(y.saturating_sub(1)),
            };
        }
        None
    }

    pub fn highlight(&mut self, word: Option<&str>) {
        for row in &mut self.rows {
            row.highlight(self.filetype.highlightling_options(), word);
        }
    }
}

impl Index<usize> for Document {
    type Output = Row;
    fn index(&self, index: usize) -> &Self::Output {
        #[allow(clippy::indexing_slicing)]
        &self.rows[index]
    }
}
