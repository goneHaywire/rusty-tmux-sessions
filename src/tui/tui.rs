use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        self, event::DisableMouseCapture, terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}
    },
    Terminal,
};
use std::io;

use super::{app::App, event::EventHandler, view};

pub struct TUI<B: Backend> {
    pub terminal: Terminal<B>,
    pub events: EventHandler,
}

impl<B> TUI<B>
where
    B: Backend,
{
    pub fn new(terminal: Terminal<B>, events: EventHandler) -> Self {
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
