// FIXME: remove this debug allow
#![allow(unused)]

use std::{
    fs::File,
    io::{stdout, Stdout},
    time::Duration,
};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
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
}

#[derive(Debug, Default)]
enum AppMode {
    #[default]
    Normal,
}

#[derive(Debug)]
enum AppAction {
    None,
    Quit,
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
        }
    }

    pub fn run(&self) -> anyhow::Result<()> {
        tui::init()?;
        init_log()?;

        let mut term = Terminal::new(CrosstermBackend::new(stdout()))?;

        loop {
            self.draw(&mut term);
            term.set_cursor(0, 0);
            term.show_cursor();

            if event::poll(Duration::from_millis(10))? {
                let event = event::read()?;
                match self.handle_event(event)? {
                    AppAction::Quit => break,
                    AppAction::None => {}
                };
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
        match event {
            Event::Key(key) => {
                debug!("{:?}", key);
                if key.code == KeyCode::Char('q') {
                    Ok(AppAction::Quit)
                } else {
                    Ok(AppAction::None)
                }
            }
            _ => todo!(),
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
