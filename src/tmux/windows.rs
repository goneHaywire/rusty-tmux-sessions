use std::{
    fmt::Display,
    str::{self, FromStr},
};

use anyhow::{Context, Error, Result};

use crate::tui::{logger::Logger, mode::CommandKind};

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
    pub session_name: String,
    is_active: bool,
    pub last_active: u64,
    pub panes_count: usize,
    pub current_command: Option<String>,
}

impl TmuxEntity for Window {}

impl FromStr for Window {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<_> = s.split(',').collect();

        assert_eq!(
            parts.len(),
            7,
            "should be 7 parts in list-windows format str"
        );

        Ok(Window {
            id: parts[0].parse().unwrap(),
            name: parts[1].into(),
            session_name: parts[2].into(),
            is_active: parts[3] == "1",
            last_active: parts[4]
                .parse()
                .context("error parsing window last_active")?,
            panes_count: parts[5]
                .parse()
                .context("error parsing window panes_count")?,
            current_command: if parts[6].is_empty() {
                None
            } else {
                Some(parts[6].into())
            },
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

    pub fn get_window(id: &IdW) -> Result<Window> {
        let window = TmuxCommand::get_window(id)?;

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
        TmuxCommand::create_window(name, id, pos)
    }

    pub fn kill(id: &IdW) -> Result<()> {
        TmuxCommand::kill_window(id)
    }

    pub fn rename(id: &IdW, new_name: &str) -> Result<()> {
        TmuxCommand::rename_window(id, new_name)
    }

    pub fn attach(id: &IdW) -> Result<()> {
        TmuxCommand::attach_window(id)
    }

    pub fn send_keys(id: &IdW, keys: &[u8], kind: CommandKind) -> Result<()> {
        let keys: &[&str] = match kind {
            CommandKind::Program => &[str::from_utf8(keys).unwrap(), "Enter"],
            CommandKind::Keys => &[str::from_utf8(keys).unwrap()],
        };
        TmuxCommand::send_keys(id, keys)
    }
}

#[test]
fn from_str() {
    let window_str = "@42,test_window,sesh,1,1722892534,4,cargo";
    let window = Window::from_str(window_str).unwrap();

    assert_eq!(IdW::from(42), window.id);
    assert_eq!("test_window".to_string(), window.name);
    assert_eq!("sesh".to_string(), window.session_name);
    assert!(window.is_active);
    assert_eq!(1722892534, window.last_active);
    assert_eq!(4, window.panes_count);
    assert_eq!("cargo".to_string(), window.current_command.unwrap());
}
