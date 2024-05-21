use std::{fs::File, io::Stdout, time::Duration};

use crossterm::event::{self, Event, KeyCode};
use log::{debug, LevelFilter};
use ratatui::{
    backend::CrosstermBackend, buffer::Buffer, layout::Rect, style::Style, widgets::Widget,
    Terminal,
};
use simplelog::{CombinedLogger, WriteLogger};

use crate::tui;

#[derive(Debug)]
pub struct App {
    mode: AppMode,
    cursor: (u16, u16),
}

#[derive(Debug, Default)]
enum AppMode {
    #[default]
    Normal,
    Insert,
    Command,
}

#[derive(Debug)]
enum AppAction {
    None,
    Quit,
    CursorMove(u16, u16),
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let test = "~";
        for row in 0..area.height {
            buf.set_string(0, row, test, Style::default());
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            mode: AppMode::Normal,
            cursor: (0, 0),
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let mut term = tui::init()?;
        init_log()?;

        loop {
            self.draw(&mut term)?;
            term.show_cursor()?;
            term.set_cursor(self.cursor.0, self.cursor.1)?;

            if event::poll(Duration::from_millis(10))? {
                let event = event::read()?;
                match self.handle_event(event)? {
                    AppAction::None => {}
                    AppAction::Quit => break,
                    AppAction::CursorMove(row, col) => {
                        self.cursor.0 = row;
                        self.cursor.1 = col;
                    }
                };
                debug!("{:?}", self);
            }
        }

        tui::restore()?;
        Ok(())
    }

    fn draw(&self, term: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
        term.draw(|frame| {
            let area = frame.size();
            frame.render_widget(self, area);
        })?;

        Ok(())
    }

    fn handle_event(&self, event: Event) -> anyhow::Result<AppAction> {
        debug!("{:?}", event);
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => Ok(AppAction::Quit),
                KeyCode::Char('h') => Ok(AppAction::CursorMove(
                    self.cursor.0.saturating_sub(1),
                    self.cursor.1,
                )),
                KeyCode::Char('j') => Ok(AppAction::CursorMove(
                    self.cursor.0,
                    self.cursor.1.saturating_add(1),
                )),
                KeyCode::Char('k') => Ok(AppAction::CursorMove(
                    self.cursor.0,
                    self.cursor.1.saturating_sub(1),
                )),
                KeyCode::Char('l') => Ok(AppAction::CursorMove(
                    self.cursor.0.saturating_add(1),
                    self.cursor.1,
                )),
                _ => Ok(AppAction::None),
            },
            _ => Ok(AppAction::None),
        }
    }
}

fn init_log() -> anyhow::Result<()> {
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Trace,
        simplelog::Config::default(),
        File::create("vix.log")?,
    )])?;

    Ok(())
}
