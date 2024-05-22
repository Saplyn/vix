use std::error::Error;

use app::App;

mod app;
mod document;
mod piece_table;
mod tui;

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::new();
    app.run()?;
    Ok(())
}
