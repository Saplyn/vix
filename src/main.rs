use app::App;

mod app;
mod piece_table;
mod tui;

fn main() -> anyhow::Result<()> {
    let mut app = App::new();
    app.run()?;
    Ok(())
}
