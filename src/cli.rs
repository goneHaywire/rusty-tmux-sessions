use core::str;
use std::{
    process::Command,
    str::{from_utf8, FromStr},
};

use anyhow::{Context, Result};

use crate::{
    session::{Session, SESSION_FORMAT},
    window::{Window, WINDOW_FORMAT},
};

pub fn list_sessions() -> Result<Vec<Session>> {
    let seshs = &Command::new("tmux")
        .args(["list-sessions", "-F", SESSION_FORMAT])
        .output()
        .context("list-sessions command failed")?
        .stdout;

    str::from_utf8(seshs)
        .context("error parsing list-sessions output")?
        .lines()
        .map(|s| s.trim())
        .map(Session::from_str)
        .collect()
}

pub fn list_windows(session_name: &str) -> Result<Vec<Window>> {
    let windows = &Command::new("tmux")
        .args(["list-windows", "-t", session_name, "-F", WINDOW_FORMAT])
        .output()
        .with_context(|| format!("list-windows failed for session {session_name}"))?
        .stdout;

    from_utf8(windows)
        .context("error parsing list-windows output")?
        .lines()
        .map(|s| s.trim())
        .map(Window::from_str)
        .collect()
}
