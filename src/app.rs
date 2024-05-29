use std::{
    cmp,
    fs::File,
    io::{self, stdout, Stdout},
    path::Path,
    time::Duration,
    u16,
};

use crossterm::{
    cursor::SetCursorStyle,
    event::{self, Event, KeyCode},
    execute,
};
use derive_tools::Display;
use log::{debug, warn, LevelFilter};
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
    cursor: Position,
    view_shift: Position,
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
    CursorViewChange {
        cursor: Position,
        view_shift: Position,
    },
    EnterMode(AppMode),
    CmdPush(char),
    CmdPop,
    CmdEnter,
    InsertChar(char),
    DeleteChar,
    BackspaceLine,
    NewLine,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Position {
    pub row: u16,
    pub col: u16,
}

#[derive(Debug, Clone, Copy)]
pub enum Move {
    None,
    Left,
    Right,
    Up,
    Down,
}

impl Position {
    pub fn free_move(self, mv: Move) -> Position {
        match mv {
            Move::Left => Position {
                row: self.row,
                col: self.col.saturating_sub(1),
            },
            Move::Up => Position {
                row: self.row.saturating_sub(1),
                col: self.col,
            },
            Move::Down => Position {
                row: self.row.saturating_add(1),
                col: self.col,
            },
            Move::Right => Position {
                row: self.row,
                col: self.col.saturating_add(1),
            },
            Move::None => Position {
                row: self.row,
                col: self.col,
            },
        }
    }
    pub fn constraint_move(self, width: u16, height: u16, mv: Move) -> Position {
        match mv {
            Move::Left => Position {
                row: self.row,
                col: self.col.saturating_sub(1),
            },
            Move::Up => Position {
                row: self.row.saturating_sub(1),
                col: self.col,
            },
            Move::Down => Position {
                row: if self.row < height {
                    self.row.saturating_add(1)
                } else {
                    self.row
                },
                col: self.col,
            },
            Move::Right => Position {
                row: self.row,
                col: if self.col < width {
                    self.col.saturating_add(1)
                } else {
                    self.col
                },
            },
            Move::None => Position {
                row: self.row,
                col: self.col,
            },
        }
    }
}

impl App {
    //~ Core Functionality

    pub fn open_file(file_path: impl AsRef<Path>) -> io::Result<Self> {
        Ok(Self {
            mode: AppMode::default(),
            cursor: Position::default(),
            view_shift: Position::default(),
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
            match self.mode {
                AppMode::Normal => execute!(stdout(), SetCursorStyle::BlinkingBlock)?,
                AppMode::Insert => execute!(stdout(), SetCursorStyle::BlinkingBar)?,
                AppMode::Command => execute!(stdout(), SetCursorStyle::SteadyUnderScore)?,
            }

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

    //~ Processing Logic

    fn process(&mut self, action: AppAction) {
        match action {
            AppAction::None => {}
            AppAction::CursorViewChange { cursor, view_shift } => {
                self.cursor.row = cursor.row;
                self.cursor.col = cursor.col;
                self.view_shift.row = view_shift.row;
                self.view_shift.col = view_shift.col;
            }
            AppAction::EnterMode(mode) => {
                if let AppMode::Command = mode {
                    self.cmd.clear()
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
            AppAction::InsertChar(ch) => {
                self.doc.insert(self.cursor, ch);
                self.cursor.col = self.cursor.col.saturating_add(1);
            }
            AppAction::DeleteChar => {
                self.doc.delete(self.cursor.free_move(Move::Left));
                self.cursor.col = self.cursor.col.saturating_sub(1);
            }
            AppAction::BackspaceLine => {
                let col = self
                    .doc
                    .get_line_len(self.cursor.row.saturating_sub(1) as usize)
                    .saturating_sub(self.view_shift.col as usize) as u16;
                self.doc.merge_line_into_up(self.cursor.row as usize);
                self.cursor.col = col;
                if self.cursor.row != 0 {
                    self.cursor.row = self.cursor.row.saturating_sub(1);
                } else {
                    self.view_shift.row = self.view_shift.row.saturating_sub(1);
                }
            }
            AppAction::NewLine => {
                self.doc.split_to_two_line(self.cursor);
                self.cursor.col = 0;
                self.cursor.row = self.cursor.row.saturating_add(1);
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

    //~ Rendering Logic

    fn draw(&self, term: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), AppError> {
        term.draw(|frame| {
            let area = frame.size();

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

            if self.show_help {
                let popup_layout = centered_rect(frame.size(), 35, 53);
                frame.render_widget(Clear, popup_layout);
                frame.render_widget(self.help_widget(), popup_layout);
            }
        })?;

        Ok(())
    }

    fn help_widget(&self) -> impl Widget {
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

    //~ Handling Event

    fn handle_event(
        &self,
        event: Event,
        term: &Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<AppAction, AppError> {
        match event {
            Event::Resize(_, _) => self.handle_event_cursor(term, Move::None),
            event => match self.mode {
                AppMode::Normal => self.handle_event_normal(event, term),
                AppMode::Insert => self.handle_event_insert(event),
                AppMode::Command => self.handle_event_command(event),
            },
        }
    }

    fn handle_event_normal(
        &self,
        event: Event,
        term: &Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<AppAction, AppError> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('h') | KeyCode::Left => self.handle_event_cursor(term, Move::Left),
                KeyCode::Char('j') | KeyCode::Down => self.handle_event_cursor(term, Move::Down),
                KeyCode::Char('k') | KeyCode::Up => self.handle_event_cursor(term, Move::Up),
                KeyCode::Char('l') | KeyCode::Right => self.handle_event_cursor(term, Move::Right),
                KeyCode::Char('i') => Ok(AppAction::EnterMode(AppMode::Insert)),
                KeyCode::Char(':') => Ok(AppAction::EnterMode(AppMode::Command)),
                _ => Ok(AppAction::None),
            },
            _ => Ok(AppAction::None),
        }
    }

    fn handle_event_cursor(
        &self,
        term: &Terminal<CrosstermBackend<Stdout>>,
        mv: Move,
    ) -> Result<AppAction, AppError> {
        let width = term.size()?.width.saturating_sub(1);
        let height = term.size()?.height.saturating_sub(2);
        let doc_height = self.doc.line_count().saturating_sub(1);

        let mut view_shift = self.view_shift;
        let mut cursor = match mv {
            Move::None => self.cursor,
            Move::Left => {
                if self.cursor.col == 0 {
                    view_shift = view_shift.free_move(Move::Left);
                    self.cursor
                } else {
                    self.cursor.free_move(Move::Left)
                }
            }
            Move::Down => self.cursor.free_move(Move::Down),
            Move::Up => {
                if self.cursor.row == 0 {
                    view_shift = view_shift.free_move(Move::Up);
                    self.cursor
                } else {
                    self.cursor.free_move(Move::Up)
                }
            }
            Move::Right => self.cursor.free_move(Move::Right),
        };

        warn!("cursor: {:?}", cursor);
        warn!("view_shift: {:?}", view_shift);

        let ln_len = self
            .doc
            .get_line_len(view_shift.row as usize + cursor.row as usize);
        let last_col = cmp::min(
            ln_len.saturating_sub(view_shift.col as usize),
            width as usize,
        );
        let last_row = cmp::min(
            doc_height.saturating_sub(view_shift.row as usize),
            height as usize,
        );

        warn!("doc_height: {:?}", doc_height);
        warn!("height: {:?}", height);
        warn!("width: {:?}", width);
        warn!("last_col: {:?}", last_col);
        warn!("last_row: {:?}", last_row);

        while cursor.col > width && (cursor.col as usize) > last_col {
            view_shift.col = view_shift.col.saturating_add(1);
            cursor.col = cursor.col.saturating_sub(1);
        }
        while cursor.row > height && (cursor.row as usize) > last_row {
            view_shift.row = view_shift.row.saturating_add(1);
            cursor.row = cursor.row.saturating_sub(1);
        }

        // horizontal
        while (cursor.col as usize).saturating_add(view_shift.col as usize) > ln_len {
            if cursor.col != 0 {
                cursor.col = cursor.col.saturating_sub(1);
            } else {
                view_shift.col = view_shift.col.saturating_sub(1);
            }
        }

        // vertical
        while (cursor.row as usize).saturating_add(view_shift.row as usize) > doc_height {
            if cursor.row != 0 {
                cursor.row = cursor.row.saturating_sub(1);
            } else {
                view_shift.row = view_shift.row.saturating_sub(1);
            }
        }

        warn!("cursor: {:?}", cursor);
        warn!("view_shift: {:?}", view_shift);

        Ok(AppAction::CursorViewChange { cursor, view_shift })
    }

    fn handle_event_insert(&self, event: Event) -> Result<AppAction, AppError> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Esc => Ok(AppAction::EnterMode(AppMode::Normal)),
                KeyCode::Char(ch) => Ok(AppAction::InsertChar(ch)),
                KeyCode::Backspace => {
                    if self.cursor.col != 0 {
                        Ok(AppAction::DeleteChar)
                    } else if self.cursor.row != 0 || self.view_shift.row != 0 {
                        Ok(AppAction::BackspaceLine)
                    } else {
                        Ok(AppAction::None)
                    }
                }
                KeyCode::Enter => Ok(AppAction::NewLine),
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
            cursor: Position::default(),
            view_shift: Position::default(),
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
