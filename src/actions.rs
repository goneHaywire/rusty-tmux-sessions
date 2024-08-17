#[derive(Debug)]
pub enum Action {
    Down,
    Up,
    Select,
    Rename,
    Kill,
    Add,
    Hide,
}

impl Action {
    pub fn from_keybind(keypress: &str) -> Option<Action> {
        match keypress {
            "<C-k>" => Some(Action::Up),
            "<C-p>" => Some(Action::Up),
            "<C-j>" => Some(Action::Down),
            "<C-n>" => Some(Action::Down),
            "<C-y>" => Some(Action::Select),
            "Enter" => Some(Action::Select),
            "<C-r>" => Some(Action::Rename),
            "<C-x>" => Some(Action::Kill),
            "<C-a>" => Some(Action::Add),
            "<C-h>" => Some(Action::Hide),
            _ => None,
        }
    }
}
