use std::ops::Index;

use ratatui::widgets::ListState;

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
#[derive(Debug)]
pub struct StatefulList {
    pub items: Vec<String>,
    pub state: ListState,
    show_hidden: bool,
}

impl Default for StatefulList {
    fn default() -> Self {
        Self {
            items: Default::default(),
            state: ListState::default().with_selected(Some(0)),
            show_hidden: false,
        }
    }
}

impl StatefulList {
    pub fn with_items(items: Vec<String>) -> Self {
        let mut list = Self::default();
        list.items(items);
        list
    }

    pub fn items(&mut self, items: Vec<String>) {
        self.items = items;
    }

    /// show or hide hidden items
    pub fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
    }

    pub fn get_active_item(&self) -> String {
        let active_idx = self
            .state
            .selected()
            .expect("there should always be a selection");
        let index = self.items.index(active_idx);
        index.clone()
    }

    /// selection function that handles 4 different cases
    ///
    /// * `selection`: Selection
    pub fn select(&mut self, selection: Selection) -> String {
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
        self.get_active_item()
    }
}
