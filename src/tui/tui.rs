use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        self,
        event::DisableMouseCapture,
        terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};
use std::io::{self, Stdout};

use super::{app::App, event::EventHandler, view};

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

pub struct TUI {
    pub terminal: Tui,
    pub events: EventHandler,
}

impl TUI {
    pub fn new(terminal: Tui, events: EventHandler) -> Self {
        Self { terminal, events }
    }

    pub fn init(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stdout(), EnterAlternateScreen)?;
        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    pub fn draw(&mut self, app: &mut App) -> io::Result<()> {
        self.terminal.draw(|frame| view::render(frame, app))?;
        Ok(())
    }

    pub fn exit(&mut self) -> io::Result<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture)?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}
