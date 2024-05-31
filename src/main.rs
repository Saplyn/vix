use std::{env, error::Error};

use app::App;

mod app;
mod document;
mod tui;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();

    let mut app = match args.len() {
        1 => App::default(),
        2 => App::open_file(&args[1])?,
        _ => panic!("not supported"),
    };

    app.run()?;
    Ok(())
}
