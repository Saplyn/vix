//! WARNING: this data structure is ASCII-only, meaning that every char that is
//! not 1 byte length will cause the program to behave incorrectly. This is
//! planed to be fixed in the near future, though.

use std::fmt::{self, Display};

#[derive(Debug)]
pub struct PieceTable {
    orig: String,
    add: String,
    pieces: Vec<PieceRecord>,
}

#[derive(Debug, Clone)]
pub struct PieceRecord {
    ty: PieceType,
    begin: usize,
    length: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum PieceType {
    Orig,
    Add,
}

impl PieceRecord {
    pub fn split(&mut self, index: usize) -> Option<PieceRecord> {
        if index == self.length {
            return None;
        }

        let length = self.length;
        self.length = index;
        Some(PieceRecord {
            ty: self.ty,
            begin: index,
            length: length - index,
        })
    }
    pub fn split_and_then(
        &mut self,
        index: usize,
        f: impl Fn(&mut PieceRecord),
    ) -> Option<PieceRecord> {
        if index == self.length {
            f(self);
            return None;
        }

        let length = self.length;
        self.length = index;
        f(self);
        Some(PieceRecord {
            ty: self.ty,
            begin: index,
            length: length - index,
        })
    }
}

impl PieceTable {
    pub fn from_string(s: String) -> Self {
        let len = s.chars().count();
        Self {
            orig: s,
            add: String::new(),
            pieces: vec![
                //~ Dummy node: Ensuring every "find" is behind a piece
                PieceRecord {
                    ty: PieceType::Orig,
                    begin: 0,
                    length: 0,
                },
                PieceRecord {
                    ty: PieceType::Orig,
                    begin: 0,
                    length: len,
                },
            ],
        }
    }

    pub fn insert_char_at(&mut self, mut char_offset: usize, ch: char) {
        let mut iter = self.pieces.iter_mut().enumerate();
        let (index, rec) = loop {
            let Some((ind, rec)) = iter.next() else {
                todo!("err handle: index out of range"); // FIXME: Index out of range
            };
            if char_offset <= rec.length {
                break (ind + 1, rec);
            }
            char_offset -= rec.length;
        };

        let begin = self.add.chars().count();
        self.add.push(ch);
        if let Some(right) = rec.split(char_offset) {
            //~ [tt]c[tt]
            self.pieces.insert(index, right);
        } //~ else: [tttt]c
        self.pieces.insert(
            index,
            PieceRecord {
                ty: PieceType::Add,
                begin,
                length: 1,
            },
        );
    }

    pub fn delete_char_at(&mut self, mut char_offset: usize) {
        let mut iter = self.pieces.iter_mut().enumerate();
        let (index, rec) = loop {
            let Some((ind, rec)) = iter.next() else {
                todo!("err handle: index out of range"); // FIXME: Index out of range
            };
            if char_offset <= rec.length {
                break (ind + 1, rec);
            }
            char_offset -= rec.length;
        };

        if let Some(right) = rec.split_and_then(char_offset, |this| this.length -= 1) {
            //~ [t_][tt]
            self.pieces.insert(index, right);
        } //~ else: [ttt_]
    }
}

impl Display for PieceTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rec in self.pieces.iter() {
            match rec.ty {
                PieceType::Orig => write!(
                    f,
                    "{}",
                    self.orig.get(rec.begin..(rec.begin + rec.length)).unwrap()
                )?,
                PieceType::Add => write!(
                    f,
                    "{}",
                    self.add.get(rec.begin..(rec.begin + rec.length)).unwrap()
                )?,
            }
        }
        Ok(())
    }
}
