mod piece_table;
mod tui;

use piece_table::vec::PieceTable;

fn main() -> anyhow::Result<()> {
    let mut tb = PieceTable::from_str("你好");
    println!("{:#?}", tb);
    println!("{}", tb);

    tb.insert(0, "i");
    println!("{:#?}", tb);
    println!("{}", tb);

    tb.delete(0, 2);
    println!("{:#?}", tb);
    println!("{}", tb);

    Ok(())
}
