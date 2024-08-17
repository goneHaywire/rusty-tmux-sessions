pub mod actions;
pub mod cli;
pub mod session;
pub mod window;

use anyhow::Result;

fn main() -> Result<()> {
    let seshs = cli::list_sessions()?;

    println!("{:#?}", seshs);
    Ok(())
}
