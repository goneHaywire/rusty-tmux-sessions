use core::str;
use std::{
    fmt::Display,
    io,
    process::{Command, Output},
};

use anyhow::{anyhow, Result};

use crate::tui::{logger::Logger, mode::CommandKind};

use super::{sessions::SessionEnv, windows::IdW};
const SESSION_FORMAT: &str =
    "#{#{session_id},#S,#{?session_attached,1,},#{session_activity},#{session_windows},#{session_created}}";

const WINDOW_FORMAT: &str =
    "#{#{window_id},#W,#S,#{window_active},#{window_activity},#{window_panes},#{pane_current_command}}";

#[derive(Default, Debug, Clone, Copy, PartialEq)]
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

fn error_decorator(_message: &str) -> String {
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

#[derive(Clone, Debug)]
pub enum TmuxCommand<'a> {
    GetSessions,
    GetWindows(&'a str),
    GetSession(&'a str),
    GetWindow(&'a IdW),
    RenameSession(&'a str, &'a str),
    RenameWindow(&'a IdW, &'a str),
    AttachSession(&'a str),
    AttachWindow(&'a IdW),
    KillSession(&'a str),
    KillWindow(&'a IdW),
    CreateSession(&'a str),
    CreateWindow(&'a str, &'a IdW, &'a WindowPos),
    SendKeys(&'a IdW, &'a [&'a str]),
    SetEnv(&'a str, SessionEnv, Option<&'a str>),
    GetEnv(&'a str, SessionEnv),
}

impl From<TmuxCommand<'_>> for Command {
    fn from(value: TmuxCommand) -> Self {
        use TmuxCommand::*;

        let mut cmd = TmuxCommand::base_cmd();

        match value {
            GetSessions => cmd.args(["list-sessions", "-F", SESSION_FORMAT]),
            GetWindows(session_name) => {
                cmd.args(["list-windows", "-t", session_name, "-F", WINDOW_FORMAT])
            }
            GetSession(name) => cmd.args([
                "list-sessions",
                "-F",
                SESSION_FORMAT,
                "-f",
                &format!("#{{m:{name},#S}}"),
            ]),
            GetWindow(id) => cmd.args([
                "list-windows",
                "-a",
                "-F",
                WINDOW_FORMAT,
                "-f",
                &format!("#{{==:{id},#{{window_id}}}}"),
            ]),
            RenameSession(old_name, new_name) => {
                cmd.args(["rename-session", "-t", old_name, new_name])
            }
            RenameWindow(id, new_name) => {
                cmd.args(["rename-window", "-t", &id.to_string(), new_name])
            }
            AttachSession(name) => cmd.args(["switch-client", "-t", name]),
            AttachWindow(id) => cmd.args(["switch-client", "-t", &id.to_string()]),
            KillSession(name) => cmd.args(["kill-session", "-t", name]),
            KillWindow(id) => cmd.args(["kill-window", "-t", &id.to_string()]),
            CreateSession(name) => cmd.args(["new-session", "-d", "-s", name]),
            CreateWindow(name, id, window_pos) => cmd.args([
                "new-window",
                "-d",
                &window_pos.to_string(),
                "-t",
                &id.to_string(),
                "-n",
                name,
            ]),
            SendKeys(window_id, keys) => cmd
                .args(["send-keys", "-t", &window_id.to_string()])
                .args(keys),
            SetEnv(session_name, key, value) => {
                cmd.args(["set-environment", "-t", session_name]);

                match value {
                    Some(value) => cmd.args([&key.to_string(), value]),
                    None => cmd.args(["-u", &key.to_string()]),
                }
            }
            GetEnv(session_name, key) => {
                cmd.args(["show-environment", "-t", session_name, &key.to_string()])
            }
        };
        cmd
    }
}

impl TmuxCommand<'_> {
    fn base_cmd() -> Command {
        let cmd = "tmux";
        Command::new(cmd)
    }

    pub fn run(self) -> Result<Vec<u8>> {
        use TmuxCommand::*;

        let mut cmd: Command = self.clone().into();

        match self {
            GetSessions => cmd.output().as_result("list-sessions command failed"),
            GetWindows(session_name) => cmd
                .output()
                .as_result(&format!("list-windows failed for session {session_name}")),
            GetSession(_) => cmd.output().as_result("get session command failed"),
            GetWindow(_) => cmd
                .output()
                .as_result("get window command failed for window @{id}"),
            RenameSession(old_name, _) => cmd
                .output()
                .as_result(&format!("rename-session failed for session {old_name}",)),
            RenameWindow(id, _) => cmd
                .output()
                .as_result(&format!("rename-window failed for window @{id}",)),
            AttachSession(name) => cmd
                .output()
                .as_result(&format!("attach-session failed for session {name}")),
            AttachWindow(id) => cmd
                .output()
                .as_result(&format!("select-window failed for window @{id}",)),
            KillSession(name) => cmd
                .output()
                .as_result(&format!("kill-session failed for session {name}",)),
            KillWindow(id) => cmd
                .output()
                .as_result(&format!("kill-window failed for window @{id}")),
            CreateSession(name) => cmd
                .output()
                .as_result(&format!("new-session failed for session {name}")),
            CreateWindow(name, _, _) => cmd
                .output()
                .as_result(&format!("new-window failed for window {name}")),
            SendKeys(_, keys) => cmd
                .output()
                .as_result(&format!("send-keys failed for keys {:?}", keys)),
            SetEnv(session_name, _, _) => cmd.output().as_result(&format!(
                "set-environment failed for session {session_name}"
            )),
            GetEnv(session_name, _) => cmd
                .output()
                .as_result(&format!(
                    "show-environment failed for session {session_name}"
                ))
                .map(|output| {
                    output
                        .as_slice()
                        .split(|char| *char == b'=')
                        .next_back()
                        .map(|v| v.trim_ascii().into())
                        .unwrap_or_default()
                }),
        }
    }
}
