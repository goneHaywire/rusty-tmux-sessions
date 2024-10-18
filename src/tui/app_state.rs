#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub enum AppState {
    #[default]
    Selecting,
    Creating,
    Deleting,
    Renaming,
    Exiting,
}

impl AppState {
    pub fn toggle_creating(self) -> Self {
        match self {
            Self::Creating => Self::Selecting,
            Self::Selecting => Self::Creating,
            v => v,
        }
    }

    pub fn toggle_deleting(self) -> Self {
        match self {
            Self::Deleting => Self::Selecting,
            Self::Selecting => Self::Deleting,
            v => v,
        }
    }

    pub fn toggle_renaming(self) -> Self {
        match self {
            Self::Renaming => Self::Selecting,
            Self::Selecting => Self::Renaming,
            v => v,
        }
    }

    pub fn exit(self) -> Self {
        match self {
            Self::Selecting => Self::Exiting,
            v => v,
        }
    }

    pub fn is_killing(&self) -> bool {
        *self == Self::Deleting
    }

    pub fn is_renaming(&self) -> bool {
        *self == Self::Renaming
    }

    pub fn is_adding(&self) -> bool {
        *self == Self::Creating
    }

    pub fn should_exit(&self) -> bool {
        *self == Self::Exiting
    }
}

#[cfg(test)]
mod test {
    use super::AppState;
    use super::AppState::*;

    #[test]
    fn toggle_creating() {
        let selecting = AppState::default();
        let creating = selecting.toggle_creating();
        assert_eq!(creating, Creating);

        let selecting = AppState::default();
        assert_eq!(selecting, Selecting);

        let mut other = Deleting;
        other = other.toggle_creating();
        assert_ne!(other, Creating);
    }

    #[test]
    fn toggle_deleting() {
        let selecting = AppState::default();
        let deleting = selecting.toggle_deleting();
        assert_eq!(deleting, Deleting);

        let selecting = AppState::default();
        assert_eq!(selecting, Selecting);

        let mut other = Creating;
        other = other.toggle_deleting();
        assert_ne!(other, Deleting);
    }

    #[test]
    fn toggle_renaming() {
        let selecting = AppState::default();
        let renaming = selecting.toggle_renaming();
        assert_eq!(renaming, Renaming);

        let selecting = AppState::default();
        assert_eq!(selecting, Selecting);

        let mut other = Creating;
        other = other.toggle_renaming();
        assert_ne!(other, Renaming);
    }

    #[test]
    fn exit() {
        let (mut selecting, mut creating, mut renaming, mut deleting) =
            (Selecting, Creating, Renaming, Deleting);
        selecting = selecting.exit();
        creating = creating.exit();
        renaming = renaming.exit();
        deleting = deleting.exit();

        assert_eq!(selecting, Exiting);
        assert_ne!(creating, Exiting);
        assert_ne!(renaming, Exiting);
        assert_ne!(deleting, Exiting);
    }
}
