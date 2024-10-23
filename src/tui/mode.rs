use super::input::InputState;

#[derive(PartialEq, Default, Clone, Copy, Debug)]
pub enum Section {
    #[default]
    Sessions,
    Windows,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Mode {
    Select(Section),
    Create(Section, InputState),
    Delete(Section),
    Rename(Section, InputState),
    Help,
    Exit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToggleResult {
    Toggled(Mode),
    NotToggled(Mode),
}

impl ToggleResult {
    pub fn unwrap(self) -> Mode {
        match self {
            Self::Toggled(mode) => mode,
            Self::NotToggled(mode) => mode,
        }
    }

    pub fn was_toggled(&self) -> bool {
        match self {
            Self::Toggled(_) => true,
            Self::NotToggled(_) => false,
        }
    }
}

use ToggleResult::*;

impl Default for Mode {
    fn default() -> Self {
        Self::Select(Section::default())
    }
}

impl Mode {
    pub fn go_to_section(&self, section: Section) -> Self {
        match self {
            Self::Select(_) => Self::Select(section),
            m => m.clone(),
        }
    }

    pub fn toggle_create(&self) -> ToggleResult {
        match self {
            Self::Create(s, _) => Toggled(Self::Select(*s)),
            Self::Select(s) => Toggled(Self::Create(*s, InputState::default())),
            v => NotToggled(v.clone()),
        }
    }

    pub fn toggle_delete(&self) -> ToggleResult {
        match self {
            Self::Delete(s) => Toggled(Self::Select(*s)),
            Self::Select(s) => Toggled(Self::Delete(*s)),
            v => NotToggled(v.clone()),
        }
    }

    pub fn toggle_rename(&self) -> ToggleResult {
        match self {
            Self::Rename(s, _) => Toggled(Self::Select(*s)),
            Self::Select(s) => Toggled(Self::Rename(*s, InputState::default())),
            v => NotToggled(v.clone()),
        }
    }

    pub fn exit(&self) -> ToggleResult {
        match self {
            Self::Select(_) => Toggled(Self::Exit),
            v => NotToggled(v.clone()),
        }
    }

    pub fn is_killing(&self) -> bool {
        match self {
            Self::Delete(_) => true,
            _ => false,
        }
    }

    pub fn is_renaming(&self) -> bool {
        match self {
            Self::Rename(_, _) => true,
            _ => false,
        }
    }

    pub fn is_adding(&self) -> bool {
        match self {
            Self::Create(_, _) => true,
            _ => false,
        }
    }

    pub fn should_exit(&self) -> bool {
        *self == Self::Exit
    }
}

#[cfg(test)]
mod test {
    use crate::tui::mode::Section;

    use super::Mode::{self, *};

    #[test]
    fn toggle_creating() {
        let selecting = Mode::default();
        let toggled = selecting.toggle_create();
        assert!(toggled.was_toggled());

        let selecting = Mode::default();
        assert_eq!(selecting, Select(Section::Sessions));

        let other = Delete(Section::Sessions);
        let not_toggled = other.toggle_create();
        assert!(!not_toggled.was_toggled());
    }

    #[test]
    fn toggle_deleting() {
        let selecting = Mode::default();
        let toggled = selecting.toggle_delete();
        assert!(toggled.was_toggled());

        let create = Mode::default().toggle_create().unwrap();
        let not_toggled = create.toggle_delete();
        assert!(!not_toggled.was_toggled());
    }

    #[test]
    fn toggle_renaming() {
        let selecting = Mode::default();
        let toggled = selecting.toggle_rename();
        assert!(toggled.was_toggled());

        let other = Delete(Section::Sessions);
        let not_toggled = other.toggle_rename();
        assert!(!not_toggled.was_toggled());
    }

    #[test]
    fn exit() {
        let (selecting, creating, renaming, deleting) = (
            Mode::default(),
            Mode::default().toggle_create().unwrap(),
            Mode::default().toggle_rename().unwrap(),
            Mode::default().toggle_delete().unwrap(),
        );
        let selecting = selecting.exit();
        let creating = creating.exit();
        let renaming = renaming.exit();
        let deleting = deleting.exit();

        assert!(selecting.was_toggled());
        assert!(!creating.was_toggled());
        assert!(!renaming.was_toggled());
        assert!(!deleting.was_toggled());
    }
}
