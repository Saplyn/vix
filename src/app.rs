use std::{fs::File, io::Stdout, time::Duration, u16};

use crossterm::event::{self, Event, KeyCode};
use derive_tools::Display;
use log::{debug, LevelFilter};
use ratatui::{
    backend::CrosstermBackend,
    style::{Style, Stylize},
    text::Line,
    Terminal,
};
use ratatui_macros::vertical;
use simplelog::{CombinedLogger, WriteLogger};
use thiserror::Error;

use crate::{document::Document, tui};

#[derive(Debug, Default)]
pub struct App {
    mode: AppMode,
    cursor: Potision,
    running: bool,
    doc: Document,
    cmd: String,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    IoErr(#[from] std::io::Error),
    #[error("{0}")]
    SetLoggerErr(#[from] log::SetLoggerError),
}

#[derive(Debug, Default, PartialEq, Eq, Display)]
enum AppMode {
    #[default]
    Normal,
    Insert,
    Command,
}

#[derive(Debug, PartialEq, Eq)]
enum AppAction {
    None,
    CursorMove(Potision),
    EnterMode(AppMode),
    CmdPush(char),
    CmdPop,
    CmdEnter,
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

impl App {
    pub fn new() -> Self {
        Self {
            mode: AppMode::Normal,
            cursor: Potision::default(),
            running: true,
            doc: Document::hello_world(),
            cmd: String::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), AppError> {
        let mut term = tui::init()?;
        init_log()?;

        while self.running {
            self.draw(&mut term)?;
            term.show_cursor()?;
            term.set_cursor(self.cursor.col, self.cursor.row)?;

            if event::poll(Duration::from_millis(10))? {
                let event = event::read()?;
                let action = self.handle_event(event, &term)?;
                debug!("{:?}", action);
                self.process(action);
                debug!("{:?}", self);
            }
        }

        tui::restore()?;
        Ok(())
    }

    fn process(&mut self, action: AppAction) {
        match action {
            AppAction::None => {}
            AppAction::CursorMove(pos) => {
                self.cursor.row = pos.row;
                self.cursor.col = pos.col;
            }
            AppAction::EnterMode(mode) => {
                match mode {
                    AppMode::Command => self.cmd.clear(),
                    _ => {}
                }
                self.mode = mode;
            }
            AppAction::CmdPop => {
                self.cmd.pop();
            }
            AppAction::CmdPush(ch) => self.cmd.push(ch),
            AppAction::CmdEnter => {
                self.process_cmd();
                self.mode = AppMode::Normal;
            }
        };
    }

    fn draw(&self, term: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), AppError> {
        term.draw(|frame| {
            let area = frame.size();
            let [main_area, status_area] = vertical![*=1, ==1].areas(area);
            frame.render_widget(&self.doc, main_area);

            let status_line = match self.mode {
                AppMode::Normal => "NORMAL".to_string(),
                AppMode::Command => format!("COMMAND: {}", self.cmd),
                AppMode::Insert => "INSERT".to_string(),
            };
            let status_style = match self.mode {
                AppMode::Normal => Style::default().bold().on_light_blue(),
                AppMode::Command => Style::default().bold().black().on_light_yellow(),
                AppMode::Insert => Style::default().bold().black().on_green(),
            };
            frame.render_widget(Line::styled(status_line, status_style), status_area);
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
        let height = term.size()?.height - 2;
        match event {
            Event::Key(key) => match key.code {
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
                KeyCode::Char(ch) => Ok(AppAction::CmdPush(ch)),
                KeyCode::Backspace => Ok(AppAction::CmdPop),
                KeyCode::Enter => Ok(AppAction::CmdEnter),
                _ => Ok(AppAction::None),
            },
            _ => Ok(AppAction::None),
        }
    }

    fn process_cmd(&mut self) {
        match self.cmd.as_str() {
            "q" | "quit" | "exit" => self.running = false,
            _ => {}
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
