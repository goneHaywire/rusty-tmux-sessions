use std::{
    io,
    process::{Command, Output},
};

use anyhow::{anyhow, Context, Result};
const SESSION_FORMAT: &str =
    "#{#S,#{?session_attached,1,},#{session_last_attached},#{session_windows},#{session_created}}";

const WINDOW_FORMAT: &str = "#{#W,#{?window_active,1,},#{window_activity},#{window_panes}}";

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
        // dbg!(&self);
        match self {
            Ok(output) => match output.status.success() {
                true => anyhow::Result::Ok(output.stdout),
                false => Err(anyhow!(msg.to_string())),
            },
            Err(_) => Err(anyhow!("command could not be run")),
        }
    }
}

pub struct TmuxCommand {}

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
            .as_result(&format!("list-windows failed for session {}", session_name))
    }

    pub fn get_session(name: &str) -> Result<Vec<u8>> {
        base_cmd()
            .args([
                "list-sessions",
                "-F",
                SESSION_FORMAT,
                "-f",
                &format!("#{{m:{},#S}}", name),
            ])
            .output()
            .as_result("get session command failed")
    }

    pub fn get_window(session_name: &str, name: &str) -> Result<Vec<u8>> {
        let mut cmd = base_cmd();
        let cmd = cmd.args([
            "list-windows",
            "-F",
            WINDOW_FORMAT,
            "-f",
            &format!("#{{m:{},#W}}", name),
            "-t",
            session_name,
        ]);
        // dbg!(&cmd.get_args());
        cmd.output().as_result("get window command failed")
    }

    pub fn rename_session(old_name: &str, new_name: &str) -> Result<()> {
        base_cmd()
            .args(["rename-session", "-t", old_name, new_name])
            .output()
            .as_result(&format!("rename-session failed for session {}", old_name))
            .map(|_| ())
    }

    pub fn rename_window(session_name: &str, old_name: &str, new_name: &str) -> Result<()> {
        base_cmd()
            .args([
                "rename-window",
                "-t",
                format!("{}:{}", session_name, old_name,).as_str(),
                new_name,
            ])
            .output()
            .as_result(&format!("rename-window failed for window {}", old_name))
            .map(|_| ())
        // dbg!(&res);
        // Ok(())
    }

    pub fn attach_session(name: &str) -> Result<()> {
        base_cmd()
            .args(["switch-client", "-t", name])
            .output()
            .as_result(&format!("attach-session failed for session {}", name))
            .map(|_| ())
    }

    pub fn attach_window(session_name: &str, name: &str) -> Result<()> {
        base_cmd()
            .args([
                "switch-client",
                "-t",
                format!("{}:{}", session_name, name).as_str(),
            ])
            .output()
            .as_result(&format!("select-window failed for window {}", name))
            .map(|_| ())
    }

    pub fn kill_session(name: &str) -> Result<()> {
        base_cmd()
            .args(["kill-session", "-t", name])
            .output()
            .as_result(&format!("kill-session failed for session {}", name))
            .map(|_| ())
    }

    pub fn kill_window(session_name: &str, name: &str) -> Result<()> {
        base_cmd()
            .args([
                "kill-window",
                "-t",
                format!("{}:{}", session_name, name).as_str(),
            ])
            .output()
            .as_result(&format!("kill-window failed for window {}", name))
            .map(|_| ())
    }

    pub fn create_session(name: &str) -> Result<()> {
        base_cmd()
            .args(["new-session", "-d", "-s", name])
            .output()
            .as_result(&format!("new-session failed for session {}", name))
            .map(|_| ())
    }

    pub fn create_window(session_name: &str, current_window_name: &str, name: &str) -> Result<()> {
        base_cmd()
            .args([
                "new-window",
                "-a",
                "-d",
                "-n",
                name,
                "-t",
                format!("{}:{}", session_name, current_window_name).as_str(),
            ])
            .output()
            .as_result(&format!("new-window failed for window {}", name))
            .map(|_| ())
    }
}
