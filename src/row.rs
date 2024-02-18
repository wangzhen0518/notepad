use std::cmp;

use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Default)]
pub struct Row {
    content: String,
    len: usize,
}

impl<T> From<T> for Row
where
    T: Into<String>,
{
    fn from(value: T) -> Self {
        let content: String = value.into();
        let len = content.graphemes(true).count(); // need to be change when editing
        Self { content, len }
    }
}

impl Row {
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.content.len());
        let start = cmp::min(start, end);
        // self.content.get(start..end).unwrap_or_default().to_string()
        self.content
            .graphemes(true)
            .skip(start)
            .take(end - start)
            .map(|c| if c == "\t" { " ".repeat(4) } else { c.into() })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    fn update_len(&mut self) {
        self.len = self.content.graphemes(true).count();
    }
}
