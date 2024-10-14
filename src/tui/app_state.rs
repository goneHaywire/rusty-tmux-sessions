#[derive(Default, PartialEq, Clone, Copy)]
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
