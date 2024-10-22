mod tmux;
mod tui;

use std::{
    io,
    process::{exit, Command},
};

use ratatui::{backend::CrosstermBackend, Terminal};
use tui::{action::Actions, app::App, event::EventHandler, handler, tui::TUI, view};

fn main() -> io::Result<()> {
    if Command::new("tmux").arg("-V").status().is_err() {
        eprintln!("Couldn't run tmux");
        exit(1);
    }

    let mut app = App::default();
    let terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let events = EventHandler::new(250);
    let mut tui = TUI::new(terminal, events);

    tui.init()?;

    while !app.mode.should_exit() {
        // draw the screen
        tui.terminal.draw(|frame| view::render(frame, &mut app))?;

        println!("got event! {:?}", tui.events.next());
        let action = match tui.events.next() {
            tui::event::Events::Tick => Actions::Tick,
            tui::event::Events::Key(k) => handler::handle_key_events(k, &mut app),
            tui::event::Events::Resize(_, _) => Actions::Tick,
        };
    }

    tui.exit()?;
    Ok(())
}
