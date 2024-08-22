use std::{
    process::Command,
    str::{self, FromStr},
};

use anyhow::{Context, Error, Result};

use super::windows;

pub const SESSION_FORMAT: &str =
    "#{#S,#{?session_attached,1,},#{session_last_attached},#{session_windows},#{session_created}}";

#[derive(Debug, Clone)]
pub struct Session {
    pub name: String,
    pub is_attached: bool,
    pub last_attached: u64,
    pub created_at: u64,
    pub windows_count: usize,
    pub windows: Vec<windows::Window>,
    pub is_hidden: bool,
}

impl FromStr for Session {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<_> = s.split(',').collect();

        assert_eq!(
            parts.len(),
            5,
            "should be 5 parts in list-sessions format str"
        );

        Ok(Session {
            name: parts[0].into(),
            is_attached: parts[1] == "1",
            last_attached: parts[2]
                .parse()
                .context("error parsing session last_attached")?,
            windows: windows::get_windows(parts[0])?,
            windows_count: parts[3]
                .parse()
                .context("error parsing session windows_count")?,
            created_at: parts[4]
                .parse()
                .context("error parsing session created_at")?,
            is_hidden: false,
        })
    }
}

fn list_sessions() -> Result<Vec<u8>> {
    Ok(Command::new("tmux")
        .args(["list-sessions", "-F", SESSION_FORMAT])
        .output()
        .context("list-sessions command failed")?
        .stdout)
}
pub fn get_sessions() -> Result<Vec<Session>> {
    let sessions = list_sessions()?;

    str::from_utf8(&sessions)
        .context("error parsing list-sessions output")?
        .lines()
        .map(|s| s.trim())
        .map(Session::from_str)
        .collect()
}
