use anyhow::{anyhow, Result};

use crate::tmux::tmux_command::WindowPos;

use super::input::InputState;

#[derive(PartialEq, Default, Clone, Copy, Debug)]
pub enum Section {
    #[default]
    Sessions,
    Windows,
}

#[derive(Default, Debug, Clone, PartialEq, Copy)]
pub enum CommandKind {
    #[default]
    Program,
    Keys,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Mode {
    Select(Section),
    Create(Section, InputState, Option<WindowPos>),
    Delete(Section),
    Rename(Section, InputState),
    SendCommand(CommandKind, InputState),
    Help,
    Exit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToggleResult {
    Toggled(Mode),
    NotToggled(Mode),
}

impl Into<Result<Mode>> for ToggleResult {
    fn into(self) -> Result<Mode> {
        match self {
            Self::Toggled(mode) => Ok(mode),
            Self::NotToggled(_) => Err(anyhow!("mode was not changed")),
        }
    }
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
    pub fn change_section(&self, section: Section) -> Self {
        match self {
            Self::Select(_) => Self::Select(section),
            Self::Delete(_) => Self::Delete(section),
            Self::Create(_, input, pos) => Self::Create(section, input.clone(), *pos),
            m => m.clone(),
        }
    }

    pub fn enter_create(&self, pos: Option<WindowPos>) -> ToggleResult {
        match self {
            Self::Select(s @ Section::Sessions) => {
                Toggled(Self::Create(*s, InputState::default(), None))
            }
            Self::Select(s @ Section::Windows) if pos.is_some() => {
                Toggled(Self::Create(*s, InputState::default(), pos))
            }
            v => NotToggled(v.clone()),
        }
    }

    pub fn enter_delete(&self) -> ToggleResult {
        match self {
            Self::Select(s) => Toggled(Self::Delete(*s)),
            v => NotToggled(v.clone()),
        }
    }

    pub fn enter_rename(&self) -> ToggleResult {
        match self {
            Self::Select(s) => Toggled(Self::Rename(*s, InputState::default())),
            v => NotToggled(v.clone()),
        }
    }

    pub fn enter_send_command(&self) -> ToggleResult {
        match self {
            Self::Select(Section::Windows) => Toggled(Self::SendCommand(
                CommandKind::default(),
                InputState::default(),
            )),
            m => NotToggled(m.clone()),
        }
    }

    pub fn change_command_mode(&self) -> ToggleResult {
        use CommandKind::*;
        use Mode::*;

        match self {
            SendCommand(Program, input_state) => Toggled(SendCommand(Keys, input_state.clone())),
            SendCommand(Keys, input_state) => Toggled(SendCommand(Program, input_state.clone())),
            m => NotToggled(m.clone()),
        }
    }

    pub fn exit_create(&self) -> ToggleResult {
        match self {
            Self::Create(s, ..) => Toggled(Self::Select(*s)),
            v => NotToggled(v.clone()),
        }
    }

    pub fn exit_delete(&self) -> ToggleResult {
        match self {
            Self::Delete(s) => Toggled(Self::Select(*s)),
            v => NotToggled(v.clone()),
        }
    }

    pub fn exit_rename(&self) -> ToggleResult {
        match self {
            Self::Rename(s, _) => Toggled(Self::Select(*s)),
            v => NotToggled(v.clone()),
        }
    }

    pub fn exit_send_command(&self) -> ToggleResult {
        match self {
            Self::SendCommand(..) => Toggled(Self::Select(Section::Windows)),
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
            Self::Create(..) => true,
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
    fn correct_toggling_create() {
        let default = Mode::default();
        assert_eq!(default, Select(Section::Sessions));

        let toggled = default.enter_create(None);
        assert!(toggled.was_toggled());

        let toggled = toggled.unwrap().exit_create();
        assert!(toggled.was_toggled());
    }

    #[test]
    fn incorrect_toggling_create() {
        let other = Delete(Section::Sessions);

        let not_toggled = other.enter_create(None);
        assert!(!not_toggled.was_toggled());

        let not_toggled = other.exit_create();
        assert!(!not_toggled.was_toggled());
    }

    #[test]
    fn correct_toggling_delete() {
        let default = Mode::default();

        let toggled = default.enter_delete();
        assert!(toggled.was_toggled());

        let toggled = toggled.unwrap().exit_delete();
        assert!(toggled.was_toggled());
    }

    #[test]
    fn incorrect_toggling_delete() {
        let create = Mode::default().enter_create(None).unwrap();

        let not_toggled = create.enter_delete();
        assert!(!not_toggled.was_toggled());

        let not_toggled = create.exit_delete();
        assert!(!not_toggled.was_toggled());
    }

    #[test]
    fn correct_toggling_rename() {
        let default = Mode::default();

        let toggled = default.enter_rename();
        assert!(toggled.was_toggled());

        let toggled = toggled.unwrap().exit_rename();
        assert!(toggled.was_toggled());
    }

    #[test]
    fn incorrect_toggling_rename() {
        let create = Mode::default().enter_create(None).unwrap();

        let not_toggled = create.enter_rename();
        assert!(!not_toggled.was_toggled());

        let not_toggled = create.exit_rename();
        assert!(!not_toggled.was_toggled());
    }

    #[test]
    fn exit() {
        let (selecting, creating, renaming, deleting) = (
            Mode::default(),
            Mode::default().enter_create(None).unwrap(),
            Mode::default().enter_rename().unwrap(),
            Mode::default().enter_delete().unwrap(),
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
