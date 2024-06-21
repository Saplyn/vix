use std::{
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
};

use thiserror::Error;

use crate::app::Position;

#[derive(Debug, Default)]
pub struct Document {
    lines: Vec<DocLine>,
    dirty: bool,
    uri: Option<PathBuf>,
}

#[derive(Debug, Default)]
pub struct DocLine {
    pub(self) content: String,
}

impl DocLine {
    pub fn from_str(ln: &str) -> Self {
        Self {
            content: String::from(ln),
        }
    }
    pub fn insert(&mut self, at: usize, ch: char) {
        if at < self.content.len() {
            self.content.insert(at, ch);
        } else {
            self.content.push(ch);
        }
    }
    pub fn delete(&mut self, at: usize) {
        if at < self.content.len() {
            self.content.remove(at);
        }
    }
}

#[derive(Debug, Error)]
pub enum DocumentError {
    #[error("{0}")]
    IoErr(#[from] io::Error),
    #[error("No URI is provided")]
    NoUri,
}

impl Document {
    #[allow(unused)]
    pub fn hello_world() -> Self {
        let lines = vec![
            DocLine::from_str("Hello World!"),
            DocLine::from_str("Hello World!"),
            DocLine::from_str("Hello World!"),
        ];
        Self {
            lines,
            dirty: true,
            uri: None,
        }
    }

    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let content = fs::read_to_string(&path)?;
        let lines = content.lines().map(DocLine::from_str).collect();
        Ok(Self {
            lines,
            dirty: false,
            uri: Some(PathBuf::from(path.as_ref())),
        })
    }

    pub fn save(&mut self) -> Result<(), DocumentError> {
        if self.uri.is_none() {
            return Err(DocumentError::NoUri);
        }

        let file = File::create(self.uri.as_ref().unwrap())?;
        let mut writer = BufWriter::new(file);
        for line in self.lines.iter() {
            writeln!(writer, "{}", line.content)?;
        }
        self.dirty = false;
        Ok(())
    }

    pub fn insert(&mut self, at: Position, ch: char) {
        self.dirty = true;
        if (at.row as usize) < self.line_count() {
            let ln = self.lines.get_mut(at.row as usize).unwrap();
            ln.insert(at.col as usize, ch);
        } else {
            let mut ln = DocLine::default();
            ln.insert(at.col as usize, ch);
            self.lines.push(ln);
        }
    }

    pub fn delete(&mut self, at: Position) {
        self.dirty = true;
        if (at.row as usize) < self.line_count() {
            let row = self.lines.get_mut(at.row as usize).unwrap();
            row.delete(at.col as usize);
        }
    }

    pub fn merge_line_into_up(&mut self, row: usize) {
        self.dirty = true;
        let line = self.lines.remove(row);
        self.lines
            .get_mut(row.saturating_sub(1))
            .unwrap()
            .content
            .push_str(&line.content);
    }

    pub fn split_to_two_line(&mut self, at: Position) {
        self.dirty = true;
        let line = self.lines.get_mut(at.row as usize).unwrap();
        let new_line = line.content.split_off(at.col as usize);
        self.lines.insert(
            at.row.saturating_add(1) as usize,
            DocLine::from_str(new_line.as_str()),
        );
    }

    pub fn set_uri(&mut self, uri: impl AsRef<Path>) {
        self.uri = Some(PathBuf::from(uri.as_ref()));
    }

    #[inline]
    pub fn dirty(&self) -> bool {
        self.dirty
    }

    #[inline]
    pub fn get_line(&self, ind: usize) -> Option<&str> {
        self.lines.get(ind).map(|ln| ln.content.as_str())
    }

    #[inline]
    pub fn get_line_len(&self, ind: usize) -> usize {
        self.lines.get(ind).map(|ln| ln.content.len()).unwrap_or(0)
    }

    #[inline]
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
}
