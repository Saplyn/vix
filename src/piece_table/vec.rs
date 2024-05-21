#![allow(unused)]

use std::fmt::{self, Display};

#[derive(Debug)]
pub struct PieceTable {
    orig: String,
    add: String,
    pieces: Vec<PieceRecord>,
}

#[derive(Debug)]
pub struct PieceRecord {
    ty: PieceType,
    beg: usize,
    len: usize,
    line_breaks: Vec<usize>,
}

#[derive(Debug, Clone, Copy)]
pub enum PieceType {
    Orig,
    Add,
}

impl PieceRecord {
    fn split(&mut self, pos: usize) -> Option<PieceRecord> {
        if pos == self.len {
            return None;
        }

        let right = match self.line_breaks.binary_search(&pos) {
            Ok(div) | Err(div) => self.line_breaks.split_off(div),
        };

        let len = self.len;
        self.len = pos;
        Some(PieceRecord {
            ty: self.ty,
            beg: pos,
            len: len - pos,
            line_breaks: right,
        })
    }

    fn split_and_then(&mut self, pos: usize, f: impl Fn(&mut PieceRecord)) -> Option<PieceRecord> {
        if pos == self.len {
            f(self);
            return None;
        }

        let right = match self.line_breaks.binary_search(&pos) {
            Ok(div) | Err(div) => self.line_breaks.split_off(div),
        };

        let len = self.len;
        self.len = pos;
        f(self);
        Some(PieceRecord {
            ty: self.ty,
            beg: pos,
            len: len - pos,
            line_breaks: right,
        })
    }
}

impl PieceTable {
    /// Create a new empty piece table.
    pub fn new() -> Self {
        PieceTable {
            orig: String::new(),
            add: String::new(),
            pieces: vec![
                // Dummy node: Ensure every record is behind a piece
                PieceRecord {
                    ty: PieceType::Orig,
                    beg: 0,
                    len: 0,
                    line_breaks: Vec::new(),
                },
            ],
        }
    }

    /// Create a new piece table from a `&str`.
    pub fn from_str(txt: &str) -> Self {
        PieceTable {
            orig: String::from(txt),
            add: String::new(),
            pieces: vec![
                // Dummy node: Ensure every record is behind a piece
                PieceRecord {
                    ty: PieceType::Orig,
                    beg: 0,
                    len: 0,
                    line_breaks: Vec::new(),
                },
                PieceRecord {
                    ty: PieceType::Orig,
                    beg: 0,
                    len: txt.chars().count(),
                    line_breaks: get_line_breaks(txt),
                },
            ],
        }
    }

    //~ Editing

    /// Insert text at the given character offset.
    pub fn insert(&mut self, mut char_offset: usize, txt: &str) {
        let (pos, rec) = {
            let mut iter = self.pieces.iter_mut().enumerate();
            loop {
                let Some((ind, rec)) = iter.next() else {
                    todo!("err handle: index out of range"); // FIXME: Index out of range
                };
                if char_offset <= rec.len {
                    break (ind + 1, rec);
                }
                char_offset -= rec.len;
            }
        };

        let mut line_breaks = get_line_breaks(txt);

        let beg = self.add.chars().count();
        let len = txt.chars().count();
        self.add.push_str(txt);
        if let Some(right) = rec.split(char_offset) {
            self.pieces.insert(pos, right);
        }
        self.pieces.insert(
            pos,
            PieceRecord {
                ty: PieceType::Add,
                beg,
                len,
                line_breaks,
            },
        )
    }

    /// Delete text at the given character offset.
    pub fn delete(&mut self, mut char_offset: usize, len: usize) {
        let (pos, rec) = {
            let mut iter = self.pieces.iter_mut().enumerate();
            loop {
                let Some((ind, rec)) = iter.next() else {
                    todo!("err handle: index out of range"); // FIXME: Index out of range
                };
                if char_offset <= rec.len {
                    break (ind + 1, rec);
                }
                char_offset -= rec.len;
            }
        };

        dbg!(rec);
        todo!("delete");
    }

    //~ Querying

    pub fn content(&self, mut char_offset: usize, len: usize) {
        todo!()
    }
    pub fn length(&self) {
        todo!()
    }
    pub fn lines_count(&self) {
        todo!()
    }
}

fn get_line_breaks(txt: &str) -> Vec<usize> {
    let mut ret: Vec<usize> = txt
        .lines()
        .map(|ln| ln.chars().count())
        .scan(0_usize, |state, x| {
            *state += x;
            Some(*state)
        })
        .collect();
    if let Some(last) = ret.pop() {
        if txt.ends_with('\n') {
            ret.push(last);
        }
    }
    ret
}

//~ EFFICIENCY: Debug only, not considered
impl Display for PieceTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rec in self.pieces.iter() {
            match rec.ty {
                PieceType::Orig => {
                    let mut s = String::new();
                    for pos in rec.beg..(rec.beg + rec.len) {
                        s.push(self.orig.chars().nth(pos).unwrap())
                    }
                    write!(f, "{}", s)?
                }
                PieceType::Add => {
                    let mut s = String::new();
                    for pos in rec.beg..(rec.beg + rec.len) {
                        s.push(self.add.chars().nth(pos).unwrap())
                    }
                    write!(f, "{}", s)?
                }
            }
        }
        Ok(())
    }
}
