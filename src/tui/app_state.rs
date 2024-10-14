#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub enum AppState {
    #[default]
    Scrolling,
    Creating,
    Deleting,
    Renaming,
    Quitting,
}

impl AppState {
    pub fn toggle_creating(self) -> Self {
        match self {
            Self::Creating => Self::Scrolling,
            Self::Scrolling => Self::Creating,
            v => v,
        }
    }

    pub fn toggle_deleting(self) -> Self {
        match self {
            Self::Deleting => Self::Scrolling,
            Self::Scrolling => Self::Deleting,
            v => v,
        }
    }

    pub fn toggle_renaming(self) -> Self {
        match self {
            Self::Renaming => Self::Scrolling,
            Self::Scrolling => Self::Renaming,
            v => v,
        }
    }

    pub fn quit(self) -> Self {
        match self {
            Self::Scrolling => Self::Quitting,
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

    pub fn is_quitting(&self) -> bool {
        *self == Self::Quitting
    }
}

#[cfg(test)]
mod test {
    use super::AppState;
    use super::AppState::*;

    #[test]
    fn toggle_creating() {
        let scrolling = AppState::default();
        let creating = scrolling.toggle_creating();
        assert_eq!(creating, Creating);

        let scrolling = AppState::default();
        assert_eq!(scrolling, Scrolling);

        let mut other = Deleting;
        other = other.toggle_creating();
        assert_ne!(other, Creating);
    }

    #[test]
    fn toggle_deleting() {
        let scrolling = AppState::default();
        let deleting = scrolling.toggle_deleting();
        assert_eq!(deleting, Deleting);

        let scrolling = AppState::default();
        assert_eq!(scrolling, Scrolling);

        let mut other = Creating;
        other = other.toggle_deleting();
        assert_ne!(other, Deleting);
    }

    #[test]
    fn toggle_renaming() {
        let scrolling = AppState::default();
        let renaming = scrolling.toggle_renaming();
        assert_eq!(renaming, Renaming);

        let scrolling = AppState::default();
        assert_eq!(scrolling, Scrolling);

        let mut other = Creating;
        other = other.toggle_renaming();
        assert_ne!(other, Renaming);
    }

    #[test]
    fn quit() {
        let (mut scrolling, mut creating, mut renaming, mut deleting) =
            (Scrolling, Creating, Renaming, Deleting);
        scrolling = scrolling.quit();
        creating = creating.quit();
        renaming = renaming.quit();
        deleting = deleting.quit();

        assert_eq!(scrolling, Quitting);
        assert_ne!(creating, Quitting);
        assert_ne!(renaming, Quitting);
        assert_ne!(deleting, Quitting);
    }
}
