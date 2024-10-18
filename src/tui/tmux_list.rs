use std::ops::Index;

use ratatui::widgets::ListState;

use crate::tmux::tmux::TmuxEntity;

pub enum Selection {
    Next,
    Prev,
    First,
    Last,
}

/// Wrapper for a stateful TUI list
///
/// * `items`: Vector of Sessions or Windows
/// * `state`: ListState
/// * `show_hidden`: Whether to show hidden items or not
pub struct StatefulList<T: TmuxEntity> {
    pub items: Vec<T>,
    pub state: ListState,
    show_hidden: bool,
}

impl<T> Default for StatefulList<T>
where
    T: TmuxEntity,
{
    fn default() -> Self {
        Self {
            items: Default::default(),
            state: ListState::default().with_selected(Some(0)),
            show_hidden: false,
        }
    }
}

impl<T> StatefulList<T>
where
    T: TmuxEntity,
{
    pub fn with_items(mut self, items: Vec<T>) -> Self {
        self.items = items;
        self
    }

    /// show or hide hidden items
    pub fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
    }

    pub fn get_active_item(&mut self) -> T {
        let active_idx = self
            .state
            .selected()
            .expect("there should always be a selection");
        self.items.index(active_idx).clone()
    }

    /// selection function that handles 4 different cases
    ///
    /// * `selection`: Selection
    pub fn select(&mut self, selection: Selection) {
        use Selection::*;
        let last_index = self.items.len() - 1;
        let current = self.state.selected().expect("invalid selection");

        match selection {
            First => self.state.select_first(),
            Last => self.state.select(Some(last_index)),
            Next if current == last_index => self.state.select_first(),
            Next => self.state.select_next(),
            Prev if current == 0 => self.state.select(Some(last_index)),
            Prev => self.state.select_previous(),
        }
    }
}
