use std::{fs, io, path::Path, slice::SliceIndex};

#[derive(Debug, Default)]
pub struct Document {
    rows: Vec<DocLine>,
}

#[derive(Debug)]
pub struct DocLine {
    content: String,
}

impl DocLine {
    pub fn from_str(ln: &str) -> Self {
        Self {
            content: String::from(ln),
        }
    }
}

impl Document {
    pub fn hello_world() -> Self {
        let rows = vec![
            DocLine::from_str("Hello World!"),
            DocLine::from_str("Hello World!"),
            DocLine::from_str("Hello World!"),
        ];
        Self { rows }
    }

    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let rows = content.lines().map(|ln| DocLine::from_str(ln)).collect();
        Ok(Self { rows })
    }

    #[inline]
    pub fn get_line(&self, ind: usize) -> Option<&str> {
        self.rows.get(ind).map(|ln| ln.content.as_str())
    }

    #[inline]
    pub fn get_line_len(&self, ind: usize) -> usize {
        self.rows.get(ind).map(|ln| ln.content.len()).unwrap_or(0)
    }

    #[inline]
    pub fn line_count(&self) -> usize {
        self.rows.len()
    }
}
