use std::str::{self, FromStr};

use anyhow::{Context, Error, Result};

use super::{
    tmux::TmuxEntity,
    tmux_command::{TmuxCommand, WindowPos},
};

#[derive(Debug, Clone, Default)]
pub struct Window {
    pub id: usize,
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
            5,
            "should be 5 parts in list-windows format str"
        );

        Ok(Window {
            id: parts[0].parse().unwrap(),
            name: parts[1].into(),
            is_active: parts[2] == "1",
            last_active: parts[3]
                .parse()
                .context("error parsing window last_active")?,
            panes_count: parts[4]
                .parse()
                .context("error parsing window panes_count")?,
        })
    }
}

pub struct WindowService;

impl WindowService {
    pub fn get_all(session_name: &str) -> Result<Vec<Window>> {
        let windows = TmuxCommand::get_windows(session_name)?;

        str::from_utf8(&windows)
            .context("error parsing list-windows output")?
            .lines()
            .map(|s| s.trim())
            .map(Window::from_str)
            .collect()
    }

    pub fn get_window(session_name: &str, id: usize) -> Result<Window> {
        let window = TmuxCommand::get_window(session_name, id)?;

        str::from_utf8(&window)
            .context("error parsing get-window output")
            .map(|s| s.trim())
            .context("error converting str to Window")
            .and_then(Window::from_str)
    }

    pub fn create(session_name: &str, id: usize, name: &str, pos: &WindowPos) -> Result<()> {
        TmuxCommand::create_window(session_name, id, name, &pos)
    }

    pub fn kill(session_name: &str, id: usize) -> Result<()> {
        TmuxCommand::kill_window(session_name, id)
    }

    pub fn rename(session_name: &str, id: usize, new_name: &str) -> Result<()> {
        TmuxCommand::rename_window(session_name, id, new_name)
    }

    pub fn attach(session_name: &str, id: usize) -> Result<()> {
        TmuxCommand::attach_window(session_name, id)
    }

    fn show(name: &str) -> Result<()> {
        todo!()
    }

    fn hide(name: &str) -> Result<()> {
        todo!()
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
