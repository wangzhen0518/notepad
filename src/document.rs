use std::{fs, ops::Index};

use crate::row::Row;

#[derive(Debug, Default)]
pub struct Document {
    rows: Vec<Row>,
}

impl Document {
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(filename)?;
        Ok(Self {
            rows: contents.lines().map(Row::from).collect(),
        })
    }

    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn row_length(&self, index: usize) -> usize {
        if let Some(row) = self.row(index) {
            row.len()
        } else {
            0
        }
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }
}

impl Index<usize> for Document {
    type Output = Row;
    fn index(&self, index: usize) -> &Self::Output {
        &self.rows[index]
    }
}
