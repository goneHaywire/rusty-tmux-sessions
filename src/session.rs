use std::str::FromStr;

use anyhow::{Context, Error, Result};

use crate::{cli, window};

pub const SESSION_FORMAT: &str =
    "#{#S,#{?session_attached,1,},#{session_last_attached},#{session_windows},#{session_created}}";

#[derive(Debug)]
pub struct Session {
    name: String,
    is_attached: bool,
    last_attached: u64,
    created_at: u64,
    windows_count: usize,
    windows: Vec<window::Window>,
    is_hidden: bool,
}

impl FromStr for Session {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<_> = s.split(',').collect();

        assert_eq!(
            parts.len(),
            5,
            "there are 5 parts in list-sessions format str"
        );

        Ok(Session {
            name: parts[0].into(),
            is_attached: parts[1] == "1",
            last_attached: parts[2]
                .parse()
                .context("error parsing session last_attached")?,
            windows: cli::list_windows(parts[0])?,
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
