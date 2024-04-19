use std::{env, fs, process::exit};

use crate::piece_table::PieceTable;

mod cli;
mod file_model;
mod piece_table;
mod tui;

// TODO: Support for full utf-8
fn main() -> anyhow::Result<()> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        eprintln!("incorrect usage: {} <filename>", &args[0]);
        exit(1);
    }

    let path = &args[1];
    eprintln!("reading file: {}", path);
    let file = fs::read_to_string(path)?;
    let mut table = PieceTable::from_string(file);

    // table.insert_char_at(10, 'a');
    // println!("{:#?}", table);
    println!("{}", table);

    // table.delete_char_at(11);
    // println!("{:#?}", table);
    // println!("{}", table);
    tui::init()?;
    tui::restore()?;

    Ok(())
}
