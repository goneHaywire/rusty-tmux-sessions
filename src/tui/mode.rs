#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub enum Mode {
    #[default]
    Select,
    Create,
    Delete,
    Rename,
    Help,
    Exit,
}

impl Mode {
    pub fn toggle_create(self) -> Self {
        match self {
            Self::Create => Self::Select,
            Self::Select => Self::Create,
            v => v,
        }
    }

    pub fn toggle_delete(self) -> Self {
        match self {
            Self::Delete => Self::Select,
            Self::Select => Self::Delete,
            v => v,
        }
    }

    pub fn toggle_rename(self) -> Self {
        match self {
            Self::Rename => Self::Select,
            Self::Select => Self::Rename,
            v => v,
        }
    }

    pub fn exit(self) -> Self {
        match self {
            Self::Select => Self::Exit,
            v => v,
        }
    }

    pub fn is_killing(&self) -> bool {
        *self == Self::Delete
    }

    pub fn is_renaming(&self) -> bool {
        *self == Self::Rename
    }

    pub fn is_adding(&self) -> bool {
        *self == Self::Create
    }

    pub fn should_exit(&self) -> bool {
        *self == Self::Exit
    }
}

#[cfg(test)]
mod test {
    use super::Mode;
    use super::Mode::*;

    #[test]
    fn toggle_creating() {
        let selecting = Mode::default();
        let creating = selecting.toggle_create();
        assert_eq!(creating, Create);

        let selecting = Mode::default();
        assert_eq!(selecting, Select);

        let mut other = Delete;
        other = other.toggle_create();
        assert_ne!(other, Create);
    }

    #[test]
    fn toggle_deleting() {
        let selecting = Mode::default();
        let deleting = selecting.toggle_delete();
        assert_eq!(deleting, Delete);

        let selecting = Mode::default();
        assert_eq!(selecting, Select);

        let mut other = Create;
        other = other.toggle_delete();
        assert_ne!(other, Delete);
    }

    #[test]
    fn toggle_renaming() {
        let selecting = Mode::default();
        let renaming = selecting.toggle_rename();
        assert_eq!(renaming, Rename);

        let selecting = Mode::default();
        assert_eq!(selecting, Select);

        let mut other = Create;
        other = other.toggle_rename();
        assert_ne!(other, Rename);
    }

    #[test]
    fn exit() {
        let (mut selecting, mut creating, mut renaming, mut deleting) =
            (Select, Create, Rename, Delete);
        selecting = selecting.exit();
        creating = creating.exit();
        renaming = renaming.exit();
        deleting = deleting.exit();

        assert_eq!(selecting, Exit);
        assert_ne!(creating, Exit);
        assert_ne!(renaming, Exit);
        assert_ne!(deleting, Exit);
    }
}
