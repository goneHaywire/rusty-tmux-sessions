use core::str;
use std::{fmt::Display, str::FromStr};

use anyhow::{Context, Error, Result};

use crate::tui::logger::Logger;

use super::{
    tmux::TmuxEntity,
    tmux_command::{TmuxCommand, WindowPos},
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, PartialOrd, Ord)]
pub struct IdW(usize);

impl From<usize> for IdW {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl FromStr for IdW {
    type Err = Error;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        let num: usize = s.trim_start_matches('@').parse()?;
        Ok(Self(num))
    }
}

impl Display for IdW {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}", self.0)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Window {
    pub id: IdW,
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

    pub fn get_window(session_name: &str, id: &IdW) -> Result<Window> {
        let window = TmuxCommand::get_window(session_name, id)?;

        str::from_utf8(&window)
            .context("error parsing get-window output")
            .map(|s| s.trim())
            .context("error converting str to Window")
            .and_then(Window::from_str)
    }

    pub fn get_last_created_window_id(session_name: &str) -> Result<IdW> {
        let windows = TmuxCommand::get_windows(session_name)?;

        let ids: Result<Vec<IdW>> = str::from_utf8(&windows)?
            .lines()
            .map(|l| {
                let (id, _) = l.split_once(',').unwrap();
                id
            })
            .map(IdW::from_str)
            .collect();
        Ok(ids.unwrap().into_iter().max().unwrap())
    }

    pub fn create(name: &str, id: &IdW, pos: &WindowPos) -> Result<()> {
        Logger::log(&format!("creating {id}"));
        TmuxCommand::create_window(name, id, pos)
    }

    pub fn kill(id: &IdW) -> Result<()> {
        Logger::log(&format!("killing {id}"));
        TmuxCommand::kill_window(id)
    }

    pub fn rename(id: &IdW, new_name: &str) -> Result<()> {
        Logger::log(&format!("renaming {id}"));
        TmuxCommand::rename_window(id, new_name)
    }

    pub fn attach(id: &IdW) -> Result<()> {
        TmuxCommand::attach_window(id)
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
