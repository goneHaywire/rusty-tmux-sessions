use std::process::Command;

use anyhow::{Context, Result};

fn base_cmd() -> Command {
    let cmd = "tmux";
    Command::new(cmd)
}

fn error_decorator(message: &str) -> String {
    todo!();
}

pub struct TmuxCommand {}

impl TmuxCommand {
    pub fn list_sessions() -> Result<Vec<u8>> {
        let session_format = "#{#S,#{?session_attached,1,},#{session_last_attached},#{session_windows},#{session_created}}";

        Ok(base_cmd()
            .args(["list-sessions", "-F", session_format])
            .output()
            .context("list-sessions command failed")?
            .stdout)
    }

    pub fn list_windows(session_name: &str) -> Result<Vec<u8>> {
        let window_format = "#{#W,#{?window_active,1,},#{window_activity},#{window_panes}}";

        Ok(base_cmd()
            .args(["list-windows", "-t", session_name, "-F", window_format])
            .output()
            .with_context(|| format!("list-windows failed for session {}", session_name))?
            .stdout)
    }

    pub fn rename_session(old_name: &str, new_name: &str) -> Result<()> {
        base_cmd()
            .args(["rename-session", "-t", old_name, new_name])
            .output()
            .with_context(|| format!("rename-session failed for session {}", old_name))?;
        Ok(())
    }

    pub fn rename_window(old_name: &str, new_name: &str) -> Result<()> {
        base_cmd()
            .args(["rename-window", "-t", old_name, new_name])
            .output()
            .with_context(|| format!("rename-window failed for window {}", old_name));
        Ok(())
    }

    pub fn attach_session(name: &str) -> Result<()> {
        base_cmd()
            .args(["switch-client", "-t", name])
            .output()
            .with_context(|| format!("attach-session failed for session {}", name))?;
        Ok(())
    }

    pub fn attach_window(name: &str) -> Result<()> {
        base_cmd()
            .args(["select-window", "-t", name])
            .output()
            .with_context(|| format!("select-window failed for window {}", name))?;
        Ok(())
    }

    pub fn kill_session(name: &str) -> Result<()> {
        base_cmd()
            .args(["kill-session", "-t", name])
            .output()
            .with_context(|| format!("kill-session failed for session {}", name));
        Ok(())
    }

    pub fn kill_window(name: &str) -> Result<()> {
        base_cmd()
            .args(["kill-window", "-t", name])
            .output()
            .with_context(|| format!("kill-window failed for window {}", name));
        Ok(())
    }

    pub fn create_session(name: &str) -> Result<()> {
        base_cmd()
            .args(["new-session", "-d", "-s", name])
            .output()
            .with_context(|| format!("new-session failed for session {}", name));
        Ok(())
    }

    pub fn create_window(current_window_name: &str, name: &str) -> Result<()> {
        base_cmd()
            .args(["new-window", "-a", current_window_name, "-n", name])
            .output()
            .with_context(|| format!("new-window failed for window {}", name));
        Ok(())
    }
}
