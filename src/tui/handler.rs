use ratatui::crossterm::event::{KeyCode, KeyEvent};

use super::{action::Actions, app::App};

pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> Actions {
    match (key_event.code, &app.section) {
        (KeyCode::Backspace, _) => todo!(),
        (KeyCode::Enter, _) => todo!(),
        (KeyCode::Left, _) => todo!(),
        (KeyCode::Right, _) => todo!(),
        (KeyCode::Up, _) => todo!(),
        (KeyCode::Down, _) => todo!(),
        (KeyCode::Home, _) => todo!(),
        (KeyCode::End, _) => todo!(),
        (KeyCode::PageUp, _) => todo!(),
        (KeyCode::PageDown, _) => todo!(),
        (KeyCode::Tab, _) => todo!(),
        (KeyCode::BackTab, _) => todo!(),
        (KeyCode::Delete, _) => todo!(),
        (KeyCode::Insert, _) => todo!(),
        (KeyCode::F(_), _) => todo!(),
        (KeyCode::Char(_), _) => todo!(),
        (KeyCode::Null, _) => todo!(),
        (KeyCode::Esc, _) => todo!(),
        (KeyCode::CapsLock, _) => todo!(),
        (KeyCode::ScrollLock, _) => todo!(),
        (KeyCode::NumLock, _) => todo!(),
        (KeyCode::PrintScreen, _) => todo!(),
        (KeyCode::Pause, _) => todo!(),
        (KeyCode::Menu, _) => todo!(),
        (KeyCode::KeypadBegin, _) => todo!(),
        (KeyCode::Media(_), _) => todo!(),
        (KeyCode::Modifier(_), _) => todo!(),
    }
}
