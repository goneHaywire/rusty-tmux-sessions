use ratatui::crossterm::event::KeyCode;

use crate::tmux::{tmux_command::WindowPos, windows::IdW};

use super::{mode::Section, tmux_list::Selection};

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

    // mode switching
    EnterCreate(Option<WindowPos>),
    ExitCreate,
    EnterRename,
    ExitRename,
    EnterDelete,
    ExitDelete,
    ToggleHelp,

    ChangeSection(Section),
    ToggleHidden,
    AttachSession,
    AttachWindow,
}
