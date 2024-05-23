use std::{
    fs::File,
    io::{self, Stdout},
    path::Path,
    time::Duration,
    u16,
};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use derive_tools::Display;
use log::{debug, LevelFilter};
use ratatui::{
    backend::CrosstermBackend,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph, Widget},
    Terminal,
};
use ratatui_macros::{line, vertical};
use simplelog::{CombinedLogger, WriteLogger};
use thiserror::Error;

use crate::{document::Document, tui};

#[derive(Debug)]
pub struct App {
    mode: AppMode,
    cursor: Potision,
    view_shift: Potision,
    show_help: bool,
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
    ViewShift(Potision),
    CursorViewChange {
        cursor: Potision,
        view_shift: Potision,
    },
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
    pub fn free_move(self, mv: Move) -> Potision {
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
                row: self.row.saturating_add(1),
                col: self.col,
            },
            Move::Right => Potision {
                row: self.row,
                col: self.col.saturating_add(1),
            },
        }
    }
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
    // LYN: Core Functionality

    pub fn open_file(file_path: impl AsRef<Path>) -> io::Result<Self> {
        Ok(Self {
            mode: AppMode::default(),
            cursor: Potision::default(),
            view_shift: Potision::default(),
            show_help: true,
            running: true,
            doc: Document::open(file_path)?,
            cmd: String::default(),
        })
    }

    pub fn run(&mut self) -> Result<(), AppError> {
        let mut term = tui::init()?;
        init_log()?;

        while self.running {
            self.draw(&mut term)?;
            term.show_cursor()?;
            term.set_cursor(self.cursor.col, self.cursor.row)?;

            if event::poll(Duration::from_millis(10))? {
                self.show_help = false;

                let event = event::read()?;
                debug!("{:?}", event);
                let action = self.handle_event(event, &term)?;
                debug!("{:?}", action);
                self.process(action);
            }
        }

        tui::restore()?;
        Ok(())
    }

    // LYN: Processing Logic

    fn process(&mut self, action: AppAction) {
        match action {
            AppAction::None => {}
            AppAction::CursorMove(pos) => {
                self.cursor.row = pos.row;
                self.cursor.col = pos.col;
            }
            AppAction::ViewShift(shift) => {
                self.view_shift.row = shift.row;
                self.view_shift.col = shift.col;
            }
            AppAction::CursorViewChange { cursor, view_shift } => {
                self.cursor.row = cursor.row;
                self.cursor.col = cursor.col;
                self.view_shift.row = view_shift.row;
                self.view_shift.col = view_shift.col;
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

    fn process_cmd(&mut self) {
        match self.cmd.as_str() {
            "q" | "quit" | "exit" => self.running = false,
            "h" | "help" => self.show_help = true,
            _ => {}
        }
    }

    fn frame_cursor(&mut self) {
        // TODO: impl or remove
    }

    // LYN: Rendering Logic

    fn draw(&self, term: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), AppError> {
        term.draw(|frame| {
            let area = frame.size();
            if self.show_help {
                let popup_layout = centered_rect(frame.size(), 35, 53);
                frame.render_widget(Clear, popup_layout);
                frame.render_widget(self.help_widgt(), popup_layout);
            }

            let [main_area, status_area] = vertical![*=1, ==1].areas(area);
            frame.render_widget(self, main_area);

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

    fn help_widgt(&self) -> impl Widget {
        let text = vec![
            line!["ViX - A Vi-like Text Editor"],
            line![],
            line![],
            line!["`:q` - to quit vix                 "],
            line!["`:h` - to display this help message"],
        ];

        Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center)
    }

    // LYN: Handling Event

    fn handle_event(
        &self,
        event: Event,
        term: &Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<AppAction, AppError> {
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
        let cursor = self.cursor;
        let view_shift = self.view_shift;

        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('h') | KeyCode::Left => {
                    Ok(if key.modifiers == KeyModifiers::CONTROL {
                        AppAction::ViewShift(view_shift.free_move(Move::Left))
                    } else {
                        AppAction::CursorMove(cursor.constraint_move(width, height, Move::Left))
                    })
                }

                KeyCode::Char('j') | KeyCode::Down => {
                    Ok(if key.modifiers == KeyModifiers::CONTROL {
                        AppAction::ViewShift(view_shift.free_move(Move::Down))
                    } else {
                        AppAction::CursorMove(cursor.constraint_move(width, height, Move::Down))
                    })
                }

                KeyCode::Char('k') | KeyCode::Up => Ok(if key.modifiers == KeyModifiers::CONTROL {
                    AppAction::ViewShift(view_shift.free_move(Move::Up))
                } else {
                    AppAction::CursorMove(cursor.constraint_move(width, height, Move::Up))
                }),

                KeyCode::Char('l') | KeyCode::Right => {
                    Ok(if key.modifiers == KeyModifiers::CONTROL {
                        AppAction::ViewShift(view_shift.free_move(Move::Right))
                    } else {
                        AppAction::CursorMove(cursor.constraint_move(width, height, Move::Right))
                    })
                }

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
}

impl Default for App {
    fn default() -> Self {
        Self {
            mode: AppMode::default(),
            cursor: Potision::default(),
            view_shift: Potision::default(),
            show_help: true,
            running: true,
            doc: Document::default(),
            cmd: String::default(),
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        for row in 0..area.height {
            if let Some(ln) = self.doc.get_line((self.view_shift.row + row) as usize) {
                if let Some(ln) = ln.get(self.view_shift.col as usize..) {
                    buf.set_string(0, row, ln, Style::default());
                } else {
                    buf.set_string(0, row, "<", Style::default().dark_gray())
                }
            } else {
                buf.set_string(0, row, "~", Style::default().dark_gray())
            }
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

// https://ratatui.rs/recipes/layout/center-a-rect/
fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
