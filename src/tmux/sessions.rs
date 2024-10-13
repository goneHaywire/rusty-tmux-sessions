use std::{
    fmt::Debug,
    str::{self, FromStr},
    usize,
};

use anyhow::{Context, Error, Result};

use super::{tmux::TmuxEntity, tmux_command::TmuxCommand};

#[derive(Debug, Clone)]
pub struct Session {
    pub name: String,
    pub is_attached: bool,
    pub last_attached: Option<u64>,
    pub created_at: u64,
    pub windows_count: usize,
    pub is_hidden: bool,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            name: Default::default(),
            is_attached: false,
            last_attached: Default::default(),
            created_at: Default::default(),
            windows_count: 0,
            is_hidden: false,
        }
    }
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
            5,
            "should be 5 parts in list-sessions format str"
        );

        let session = Session {
            name: parts[0].into(),
            is_attached: parts[1] == "1",
            last_attached: parts[2].parse::<u64>().ok(),
            windows_count: parts[3]
                .parse()
                .context("error parsing session windows_count")?,
            created_at: parts[4]
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
        let sessions = TmuxCommand::list_sessions()?;

        str::from_utf8(&sessions)
            .context("error parsing list-sessions output")?
            .lines()
            .map(|s| s.trim())
            .map(Session::from_str)
            .collect()
    }

    pub fn create(name: &str) -> Result<()> {
        TmuxCommand::create_session(name)
        //let sessions = Self::get_all("")?
        //    .iter()
        //    .find(|session| session.name == name)
        //    .expect("the newly created session should be found");
        //let session = Session::default().with_name(name);
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
        todo!();
    }

    pub fn show(name: &str) -> Result<()> {
        todo!();
    }
}
