use std::{
    io::{self, stdout, Stdout},
    panic,
};

use crossterm::{
    cursor, execute,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use ratatui::{backend::CrosstermBackend, Terminal};

pub fn init() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    let hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = restore();
        hook(panic_info);
    }));

    enable_raw_mode()?;
    execute!(stdout(), terminal::EnterAlternateScreen)?;
    execute!(stdout(), cursor::SavePosition)?;
    execute!(stdout(), cursor::EnableBlinking)?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

pub fn restore() -> io::Result<()> {
    execute!(stdout(), cursor::DisableBlinking)?;
    execute!(stdout(), cursor::RestorePosition)?;
    execute!(stdout(), terminal::LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
