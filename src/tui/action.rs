use ratatui::crossterm::event::KeyCode;

use crate::tmux::{tmux_command::WindowPos, windows::IdW};

use super::{
    mode::{CommandKind, Section},
    tmux_list::Selection,
};

pub enum Actions<'a> {
    Tick,
    Init,
    Quit,

    // helpers
    LoadSessions,
    LoadWindows,
    ClearInput,
    InputKey(KeyCode),

    // actions
    Create(Section, &'a str, Option<WindowPos>),
    Select(Section, Selection),
    Kill(Section),
    RemoveSession(String),
    RemoveWindow(String, IdW),
    Rename(Section, &'a str),
    SendCommand(CommandKind, String),

    // mode switching
    EnterCreate(Option<WindowPos>),
    ExitCreate,
    EnterRename,
    ExitRename,
    EnterDelete,
    ExitDelete,
    ToggleHelp,
    EnterSendCommand,
    ExitSendCommand,

    ChangeSection(Section),
    ToggleHidden,
    AttachSession,
    AttachWindow,
}
