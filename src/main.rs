mod tmux;
mod tui;

use std::{
    io,
    process::{exit, Command},
};

use tui::models::App;

fn main() -> io::Result<()> {
    if Command::new("tmux").arg("-V").status().is_err() {
        eprintln!("Couldn't run tmux");
        exit(1);
    }

    let mut terminal = tui::main::init()?;
    let app_result = App::default().run(&mut terminal);
    tui::main::restore()?;

    app_result
}
