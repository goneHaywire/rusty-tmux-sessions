mod tmux;
mod tui;

use std::{
    io,
    process::{exit, Command},
};

use ratatui::{backend::CrosstermBackend, Terminal};
use tui::{
    app::App,
    event::EventHandler,
    tui::{Tui, TUI},
};

fn main() -> io::Result<()> {
    if Command::new("tmux").arg("-V").status().is_err() {
        eprintln!("Couldn't run tmux");
        exit(1);
    }

    let terminal: Tui = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let events = EventHandler::new(250);
    let mut tui = TUI::new(terminal, events);

    tui.init()?;

    let result = App::default().run(&mut tui);

    tui.exit()?;
    result
}
