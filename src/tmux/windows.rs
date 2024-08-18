use std::{
    process::Command,
    str::{self, FromStr},
};

use anyhow::{Context, Error, Result};

pub const WINDOW_FORMAT: &str = "#{#W,#{?window_active,1,},#{window_activity},#{window_panes}}";

#[derive(Debug, Clone)]
pub struct Window {
    name: String,
    is_active: bool,
    last_active: u64,
    //panes: Vec<String>, TBD
    panes_count: usize,
}

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
            //panes:
            panes_count: parts[3]
                .parse()
                .context("error parsing window panes_count")?,
        })
    }
}

fn list_windows(session_name: &str) -> Result<Vec<u8>> {
    Ok(Command::new("tmux")
        .args(["list-windows", "-t", session_name, "-F", WINDOW_FORMAT])
        .output()
        .with_context(|| format!("list-windows failed for session {session_name}"))?
        .stdout)
}

pub fn get_windows(session_name: &str) -> Result<Vec<Window>> {
    let windows = list_windows(session_name)?;

    str::from_utf8(&windows)
        .context("error parsing list-windows output")?
        .lines()
        .map(|s| s.trim())
        .map(Window::from_str)
        .collect()
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
