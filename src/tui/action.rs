use ratatui::crossterm::event::KeyCode;

use super::{mode::Section, tmux_list::Selection};

pub enum Actions<'a> {
    Tick,
    Init,
    Quit,
    LoadSessions,
    LoadWindows,
    CreateSession(&'a str),
    CreateWindow(&'a str),
    SelectSession(Selection),
    SelectWindow(Selection),
    KillSession,
    KillWindow,
    RenameSession(&'a str),
    RenameWindow(&'a str),
    ToggleRename,
    ToggleCreate,
    ToggleDelete,
    ToggleHelp,
    ChangeSection(Section),
    ClearInput,
    InputKey(KeyCode),
    ToggleHidden,
    AttachSession,
    AttachWindow,
}
