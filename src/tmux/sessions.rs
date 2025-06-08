use core::str;
use std::{fmt::Display, str::FromStr, thread::scope};

use anyhow::{Context, Error, Result};

use super::tmux_command::TmuxCommand;

#[derive(Clone, Debug)]
pub enum SessionEnv {
    Hidden,
}

impl Display for SessionEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionEnv::Hidden => write!(f, "HIDDEN"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Session {
    pub id: usize,
    pub name: String,
    pub is_attached: bool,
    pub last_attached: Option<u64>,
    pub created_at: u64,
    windows_count: usize,
    pub is_hidden: bool,
}

impl FromStr for Session {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<_> = s.split(',').collect();
        let mut is_hidden = false;

        assert_eq!(
            parts.len(),
            6,
            "should be 6 parts in list-sessions format str"
        );

        scope(|s| {
            s.spawn(|| {
                is_hidden = TmuxCommand::GetEnv(parts[1], SessionEnv::Hidden)
                    .run()
                    .and_then(|v| String::from_utf8(v).map_err(Into::into))
                    .map(|v| v == "1")
                    .unwrap_or_default();
            });
        });

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
            is_hidden,
        };
        Ok(session)
    }
}

pub struct SessionService;

impl SessionService {
    pub fn get_all() -> Result<Vec<Session>> {
        let sessions = TmuxCommand::GetSessions.run()?;

        str::from_utf8(&sessions)
            .context("error parsing list-sessions output")?
            .lines()
            .map(|s| s.trim())
            .map(Session::from_str)
            .collect()
    }

    pub fn get_session(name: &str) -> Result<Session> {
        let session = TmuxCommand::GetSession(name).run()?;

        str::from_utf8(&session)
            .context("error parsing get session output")
            .map(|s| s.trim())
            .and_then(Session::from_str)
    }

    pub fn create(name: &str) -> Result<()> {
        TmuxCommand::CreateSession(name).run().map(|_| ())
    }

    pub fn kill(name: &str) -> Result<()> {
        TmuxCommand::KillSession(name).run().map(|_| ())
    }

    pub fn rename(old_name: &str, new_name: &str) -> Result<()> {
        TmuxCommand::RenameSession(old_name, new_name)
            .run()
            .map(|_| ())
    }

    pub fn attach(name: &str) -> Result<()> {
        TmuxCommand::AttachSession(name).run().map(|_| ())
    }

    pub fn hide(name: &str) -> Result<()> {
        TmuxCommand::SetEnv(name, SessionEnv::Hidden, Some("1"))
            .run()
            .map(|_| ())
    }

    pub fn show(name: &str) -> Result<()> {
        TmuxCommand::SetEnv(name, SessionEnv::Hidden, None)
            .run()
            .map(|_| ())
    }
}
