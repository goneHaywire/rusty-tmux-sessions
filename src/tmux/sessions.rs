use std::{
    fmt::Debug,
    str::{self, FromStr},
};

use anyhow::{Context, Error, Result};

use super::{tmux::TmuxEntity, tmux_command::TmuxCommand};

#[derive(Debug, Clone, Default)]
pub struct Session {
    pub id: usize,
    pub name: String,
    is_attached: bool,
    pub last_attached: Option<u64>,
    created_at: u64,
    windows_count: usize,
    is_hidden: bool,
}

impl Session {
    fn with_name(mut self, name: &str) -> Self {
        self.name = name.into();
        self
    }
}

impl TmuxEntity for Session {}

impl FromStr for Session {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<_> = s.split(',').collect();

        assert_eq!(
            parts.len(),
            6,
            "should be 6 parts in list-sessions format str"
        );

        let session = Session {
            id: parts[0].trim_start_matches('$').parse().unwrap(),
            name: parts[1].into(),
            is_attached: parts[2] == "1",
            last_attached: parts[3].parse::<u64>().ok(),
            windows_count: parts[4]
                .parse()
                .context("error parsing session windows_count")?,
            created_at: parts[5]
                .parse()
                .context("error parsing session created_at")?,
            is_hidden: false,
        };
        Ok(session)
    }
}

pub struct SessionService;

impl SessionService {
    pub fn get_all() -> Result<Vec<Session>> {
        let sessions = TmuxCommand::get_sessions()?;

        str::from_utf8(&sessions)
            .context("error parsing list-sessions output")?
            .lines()
            .map(|s| s.trim())
            .map(Session::from_str)
            .collect()
    }

    pub fn get_session(name: &str) -> Result<Session> {
        let session = TmuxCommand::get_session(name)?;

        str::from_utf8(&session)
            .context("error parsing get session output")
            .map(|s| s.trim())
            .and_then(Session::from_str)
    }

    pub fn create(name: &str) -> Result<()> {
        TmuxCommand::create_session(name)
    }

    pub fn kill(name: &str) -> Result<()> {
        TmuxCommand::kill_session(name)
    }

    pub fn rename(old_name: &str, new_name: &str) -> Result<()> {
        TmuxCommand::rename_session(old_name, new_name)
    }

    pub fn attach(name: &str) -> Result<()> {
        TmuxCommand::attach_session(name)
    }

    pub fn hide(name: &str) -> Result<()> {
        todo!()
    }

    pub fn show(name: &str) -> Result<()> {
        todo!()
    }
}
