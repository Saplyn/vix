use crate::piece_table::PieceTable;

mod piece_table;

fn main() {
    let mut table = PieceTable::from_string("0123456789".to_string());

    table.insert_char_at(10, 'a');
    println!("{:#?}", table);
    println!("{}", table);

    table.delete_char_at(11);
    println!("{:#?}", table);
    println!("{}", table);
}
