```rust
impl PieceTable {
    //~ Basics
    pub fn new() -> Self {}
    pub fn from_str(path: impl AsRef<str>) -> Self {}
    //~ Editing
    pub fn insert(&mut self, pos: usize, txt: impl AsRef<str>) {}
    pub fn delete(&mut self, pos: usize, len: usize) {}
    //~ Querying
   	pub fn content(&self, pos: usize, len: usize) {}
    pub fn length(&self) {}
    pub fn lines_count(&self) {}
    //~ Utilities
    pub fn lines<'tb>(&'tb self) -> Lines<'tb> {}
    pub fn pieces<'tb>(&'tb self) -> Pieces<'tb> {}
}
```

