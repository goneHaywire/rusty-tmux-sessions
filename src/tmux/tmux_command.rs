use std::process::Command;

use anyhow::{Context, Result};
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

pub struct TmuxCommand {}

impl TmuxCommand {
    pub fn get_sessions() -> Result<Vec<u8>> {
        Ok(base_cmd()
            .args(["list-sessions", "-F", SESSION_FORMAT])
            .output()
            .context("list-sessions command failed")?
            .stdout)
    }

    pub fn get_windows(session_name: &str) -> Result<Vec<u8>> {
        Ok(base_cmd()
            .args(["list-windows", "-t", session_name, "-F", WINDOW_FORMAT])
            .output()
            .with_context(|| format!("list-windows failed for session {}", session_name))?
            .stdout)
    }

    pub fn get_session(name: &str) -> Result<Vec<u8>> {
        Ok(base_cmd()
            .args([
                "list-sessions",
                "-F",
                SESSION_FORMAT,
                "-f",
                &format!("#{{m:{},#S}}", name),
            ])
            .output()
            .context("get session command failed")?
            .stdout)
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
        dbg!(&cmd.get_args());
        Ok(cmd.output().context("get window command failed")?.stdout)
    }

    pub fn rename_session(old_name: &str, new_name: &str) {
        let _ = base_cmd()
            .args(["rename-session", "-t", old_name, new_name])
            .output()
            .with_context(|| format!("rename-session failed for session {}", old_name));
    }

    pub fn rename_window(session_name: &str, old_name: &str, new_name: &str) {
        let _ = base_cmd()
            .args([
                "rename-window",
                "-t",
                format!("{}:{}", session_name, old_name,).as_str(),
                new_name,
            ])
            .output()
            .with_context(|| format!("rename-window failed for window {}", old_name));
    }

    pub fn attach_session(name: &str) {
        let _ = base_cmd()
            .args(["switch-client", "-t", name])
            .output()
            .with_context(|| format!("attach-session failed for session {}", name));
    }

    pub fn attach_window(session_name: &str, name: &str) {
        let _ = base_cmd()
            .args([
                "switch-client",
                "-t",
                format!("{}:{}", session_name, name).as_str(),
            ])
            .output()
            .with_context(|| format!("select-window failed for window {}", name));
    }

    pub fn kill_session(name: &str) {
        let _ = base_cmd()
            .args(["kill-session", "-t", name])
            .output()
            .with_context(|| format!("kill-session failed for session {}", name));
    }

    pub fn kill_window(session_name: &str, name: &str) {
        let _ = base_cmd()
            .args([
                "kill-window",
                "-t",
                format!("{}:{}", session_name, name).as_str(),
            ])
            .output()
            .with_context(|| format!("kill-window failed for window {}", name));
    }

    pub fn create_session(name: &str) {
        let _ = base_cmd()
            .args(["new-session", "-d", "-s", name])
            .output()
            .with_context(|| format!("new-session failed for session {}", name));
    }

    pub fn create_window(session_name: &str, current_window_name: &str, name: &str) {
        let _ = base_cmd()
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
            .with_context(|| format!("new-window failed for window {}", name));
    }
}
