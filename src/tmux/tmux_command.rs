use std::{
    fmt::Display,
    io,
    process::{Command, Output},
};

use anyhow::{anyhow, Result};
const SESSION_FORMAT: &str =
    "#{#{session_id},#S,#{?session_attached,1,},#{session_last_attached},#{session_windows},#{session_created}}";

const WINDOW_FORMAT: &str =
    "#{#{window_id},#W,#{window_active},#{window_activity},#{window_panes}}";

#[derive(Default)]
pub enum WindowPos {
    Before,
    #[default]
    After,
}

impl Display for WindowPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            WindowPos::Before => "-b",
            WindowPos::After => "-a",
        })
    }
}

fn base_cmd() -> Command {
    let cmd = "tmux";
    Command::new(cmd)
}

fn error_decorator(message: &str) -> String {
    todo!();
}

trait IoToAnyhowResult {
    fn as_result(self, msg: &str) -> Result<Vec<u8>>;
}

impl IoToAnyhowResult for io::Result<Output> {
    fn as_result(self, msg: &str) -> Result<Vec<u8>> {
        match self {
            Ok(output) => match output.status.success() {
                true => anyhow::Result::Ok(output.stdout),
                false => Err(anyhow!(msg.to_string())),
            },
            Err(_) => Err(anyhow!("command could not be run")),
        }
    }
}

pub struct TmuxCommand;

impl TmuxCommand {
    pub fn get_sessions() -> Result<Vec<u8>> {
        base_cmd()
            .args(["list-sessions", "-F", SESSION_FORMAT])
            .output()
            .as_result("list-sessions command failed")
    }

    pub fn get_windows(session_name: &str) -> Result<Vec<u8>> {
        base_cmd()
            .args(["list-windows", "-t", session_name, "-F", WINDOW_FORMAT])
            .output()
            .as_result(&format!("list-windows failed for session {session_name}",))
    }

    pub fn get_session(name: &str) -> Result<Vec<u8>> {
        base_cmd()
            .args([
                "list-sessions",
                "-F",
                SESSION_FORMAT,
                "-f",
                &format!("#{{m:{name},#S}}"),
            ])
            .output()
            .as_result("get session command failed")
    }

    pub fn get_window(session_name: &str, index: usize) -> Result<Vec<u8>> {
        base_cmd()
            .args([
                "list-windows",
                "-F",
                WINDOW_FORMAT,
                "-f",
                &format!("#{{m:{index},#I}}"),
                "-t",
                session_name,
            ])
            .output()
            .as_result("get window command failed for window at index {index}")
    }

    pub fn rename_session(old_name: &str, new_name: &str) -> Result<()> {
        base_cmd()
            .args(["rename-session", "-t", old_name, new_name])
            .output()
            .as_result(&format!("rename-session failed for session {old_name}",))
            .map(|_| ())
    }

    pub fn rename_window(session_name: &str, index: usize, new_name: &str) -> Result<()> {
        base_cmd()
            .args([
                "rename-window",
                "-t",
                &format!("{session_name}:{index}"),
                new_name,
            ])
            .output()
            .as_result(&format!("rename-window failed for window at index {index}",))
            .map(|_| ())
    }

    pub fn attach_session(name: &str) -> Result<()> {
        base_cmd()
            .args(["switch-client", "-t", name])
            .output()
            .as_result(&format!("attach-session failed for session {name}"))
            .map(|_| ())
    }

    pub fn attach_window(session_name: &str, index: usize) -> Result<()> {
        base_cmd()
            .args(["switch-client", "-t", &format!("{session_name}:{index}")])
            .output()
            .as_result(&format!("select-window failed for window at index {index}",))
            .map(|_| ())
    }

    pub fn kill_session(name: &str) -> Result<()> {
        base_cmd()
            .args(["kill-session", "-t", name])
            .output()
            .as_result(&format!("kill-session failed for session {name}",))
            .map(|_| ())
    }

    pub fn kill_window(session_name: &str, index: usize) -> Result<()> {
        base_cmd()
            .args(["kill-window", "-t", &format!("{session_name}:{index}")])
            .output()
            .as_result(&format!("kill-window failed for window {index}"))
            .map(|_| ())
    }

    pub fn create_session(name: &str) -> Result<()> {
        base_cmd()
            .args(["new-session", "-d", "-s", name])
            .output()
            .as_result(&format!("new-session failed for session {name}"))
            .map(|_| ())
    }

    pub fn create_window(
        session_name: &str,
        index: usize,
        name: &str,
        pos: &WindowPos,
    ) -> Result<()> {
        base_cmd()
            .args([
                "new-window",
                "-d",
                &pos.to_string(),
                "-n",
                name,
                "-t",
                &format!("{session_name}:{index}"),
            ])
            .output()
            .as_result(&format!("new-window failed for window {name}"))
            .map(|_| ())
    }
}
