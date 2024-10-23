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
    // while !app.mode.should_exit() {
    //     // draw the screen
    //     tui.terminal.draw(|frame| view::render(frame, &mut app))?;
    //
    //     println!("got event! {:?}", tui.events.next());
    //     let action = match tui.events.next() {
    //         tui::event::Events::Tick => Actions::Tick,
    //         tui::event::Events::Key(k) => handler::handle_key_events(k, &mut app),
    //         tui::event::Events::Resize(_, _) => Actions::Tick,
    //     };
    // }

    tui.exit()?;
    result
}
