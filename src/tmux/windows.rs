use std::str::{self, FromStr};

use anyhow::{Context, Error, Result};

use super::{tmux::TmuxEntity, tmux_command::TmuxCommand};

#[derive(Debug, Clone)]
pub struct Window {
    pub name: String,
    is_active: bool,
    last_active: u64,
    panes_count: usize,
}

impl TmuxEntity for Window {}

impl FromStr for Window {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<_> = s.split(',').collect();

        assert_eq!(
            parts.len(),
            4,
            "should be 4 parts in list-windows format str"
        );

        Ok(Window {
            name: parts[0].into(),
            is_active: parts[1] == "1",
            last_active: parts[2]
                .parse()
                .context("error parsing window last_active")?,
            panes_count: parts[3]
                .parse()
                .context("error parsing window panes_count")?,
        })
    }
}

pub struct WindowService;

impl WindowService {
    pub fn get_all(session_name: &str) -> Result<Vec<Window>> {
        //let windows = WindowService::list(Some(session_name))?;
        //let session = session.expect("no session passed to WindowService::get_all");
        let windows = TmuxCommand::list_windows(session_name)?;

        str::from_utf8(&windows)
            .context("error parsing list-windows output")?
            .lines()
            .map(|s| s.trim())
            .map(Window::from_str)
            .collect()
    }

    fn create(current_window_name: &str, name: &str) -> Result<()> {
        TmuxCommand::create_window(current_window_name, name)
    }

    fn kill(name: &str) -> Result<()> {
        TmuxCommand::kill_window(name)
    }

    fn rename(old_name: &str, new_name: &str) -> Result<()> {
        TmuxCommand::rename_window(old_name, new_name)
    }

    fn attach(name: &str) -> Result<()> {
        TmuxCommand::attach_window(name)
    }

    fn show(name: &str) -> Result<()> {
        todo!();
    }

    fn hide(name: &str) -> Result<()> {
        todo!();
    }
}

#[test]
fn from_str() {
    let window_str = "test_window,1,1722892534,4";
    let window = Window::from_str(window_str).unwrap();

    assert_eq!("test_window".to_string(), window.name);
    assert!(window.is_active);
    assert_eq!(1722892534, window.last_active);
    assert_eq!(4, window.panes_count);
}
