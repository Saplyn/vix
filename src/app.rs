use std::{fs::File, io::Stdout, time::Duration};

use crossterm::event::{self, Event, KeyCode};
use log::{debug, LevelFilter};
use ratatui::{
    backend::CrosstermBackend, buffer::Buffer, layout::Rect, style::Style, widgets::Widget,
    Terminal,
};
use simplelog::{CombinedLogger, WriteLogger};
use thiserror::Error;

use crate::tui;

#[derive(Debug)]
pub struct App {
    mode: AppMode,
    cursor: Potision,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    IoErr(#[from] std::io::Error),
    #[error("{0}")]
    SetLoggerErr(#[from] log::SetLoggerError),
}

#[derive(Debug, Default, PartialEq, Eq)]
enum AppMode {
    #[default]
    Normal,
    Insert,
    Command,
}

#[derive(Debug, PartialEq, Eq)]
enum AppAction {
    None,
    Quit,
    CursorMove(Potision),
    EnterMode(AppMode),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Potision {
    row: u16,
    col: u16,
}

#[derive(Debug, Clone, Copy)]
pub enum Move {
    Left,
    Right,
    Up,
    Down,
}

impl Potision {
    pub fn constraint_move(self, width: u16, height: u16, mv: Move) -> Potision {
        match mv {
            Move::Left => Potision {
                row: self.row,
                col: self.col.saturating_sub(1),
            },
            Move::Up => Potision {
                row: self.row.saturating_sub(1),
                col: self.col,
            },
            Move::Down => Potision {
                row: if self.row < height {
                    self.row.saturating_add(1)
                } else {
                    self.row
                },
                col: self.col,
            },
            Move::Right => Potision {
                row: self.row,
                col: if self.col < width {
                    self.col.saturating_add(1)
                } else {
                    self.col
                },
            },
        }
    }
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
            cursor: Potision::default(),
        }
    }

    pub fn run(&mut self) -> Result<(), AppError> {
        let mut term = tui::init()?;
        init_log()?;

        loop {
            self.draw(&mut term)?;
            term.show_cursor()?;
            term.set_cursor(self.cursor.col, self.cursor.row)?;

            if event::poll(Duration::from_millis(10))? {
                let event = event::read()?;
                match self.handle_event(event, &term)? {
                    AppAction::None => {}
                    AppAction::Quit => break,
                    AppAction::CursorMove(pos) => {
                        self.cursor.row = pos.row;
                        self.cursor.col = pos.col;
                    }
                    AppAction::EnterMode(mode) => {
                        self.mode = mode;
                    }
                };
                debug!("{:?}", self);
            }
        }

        tui::restore()?;
        Ok(())
    }

    fn draw(&self, term: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), AppError> {
        term.draw(|frame| {
            let area = frame.size();
            frame.render_widget(self, area);
        })?;

        Ok(())
    }

    fn handle_event(
        &self,
        event: Event,
        term: &Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<AppAction, AppError> {
        debug!("{:?}", event);
        match self.mode {
            AppMode::Normal => self.handle_event_normal(event, term),
            AppMode::Insert => self.handle_event_insert(event),
            AppMode::Command => self.handle_event_command(event),
        }
    }

    fn handle_event_normal(
        &self,
        event: Event,
        term: &Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<AppAction, AppError> {
        let width = term.size()?.width - 1;
        let height = term.size()?.height - 1;
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => Ok(AppAction::Quit),
                KeyCode::Char('h') | KeyCode::Left => Ok(AppAction::CursorMove(
                    self.cursor.constraint_move(width, height, Move::Left),
                )),
                KeyCode::Char('j') | KeyCode::Down => Ok(AppAction::CursorMove(
                    self.cursor.constraint_move(width, height, Move::Down),
                )),
                KeyCode::Char('k') | KeyCode::Up => Ok(AppAction::CursorMove(
                    self.cursor.constraint_move(width, height, Move::Up),
                )),
                KeyCode::Char('l') | KeyCode::Right => Ok(AppAction::CursorMove(
                    self.cursor.constraint_move(width, height, Move::Right),
                )),
                KeyCode::Char('i') => Ok(AppAction::EnterMode(AppMode::Insert)),
                KeyCode::Char(':') => Ok(AppAction::EnterMode(AppMode::Command)),
                _ => Ok(AppAction::None),
            },
            _ => Ok(AppAction::None),
        }
    }

    fn handle_event_insert(&self, event: Event) -> Result<AppAction, AppError> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Esc => Ok(AppAction::EnterMode(AppMode::Normal)),
                _ => Ok(AppAction::None),
            },
            _ => Ok(AppAction::None),
        }
    }

    fn handle_event_command(&self, event: Event) -> Result<AppAction, AppError> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Esc => Ok(AppAction::EnterMode(AppMode::Normal)),
                _ => Ok(AppAction::None),
            },
            _ => Ok(AppAction::None),
        }
    }
}

fn init_log() -> Result<(), AppError> {
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Trace,
        simplelog::Config::default(),
        File::create("vix.log")?,
    )])?;

    Ok(())
}
