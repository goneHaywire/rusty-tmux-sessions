use std::ops::Index;

use ratatui::widgets::ListState;

#[derive(PartialEq, Copy, Clone)]
pub enum Selection {
    Index(Option<usize>),
    NextNoWrap,
    PrevNoWrap,
    Next,
    Prev,
    First,
    Last,
    Noop,
}

/// Wrapper for a stateful TUI list
///
/// * `items`: Vector of Sessions or Windows
/// * `state`: ListState
#[derive(Debug)]
pub struct StatefulList {
    pub items: Vec<String>,
    pub state: ListState,
}

impl Default for StatefulList {
    fn default() -> Self {
        Self {
            items: Default::default(),
            state: ListState::default().with_selected(Some(0)),
        }
    }
}

impl StatefulList {
    /// replace the list items and select the last item if the previous selection exceeded the new
    /// items length
    ///
    /// * `items`: new list to replace the old
    pub fn set_items(&mut self, items: Vec<String>) {
        self.items = items;
        if let Some(index) = self.state.selected() {
            if index >= self.items.len() {
                self.select(Selection::Last);
            }
        }
    }

    pub fn get_active_item(&self) -> &String {
        let active_idx = self
            .state
            .selected()
            .expect("there should always be a selection");
        self.items.index(active_idx)
    }

    /// selection function that handles multiple selection options and returns the selected &item
    ///
    /// * `selection`: Selection
    pub fn select(&mut self, selection: Selection) -> &String {
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
            Index(index) => self.state.select(index),
            NextNoWrap => self.state.select_next(),
            PrevNoWrap => self.state.select_previous(),
            Noop => (),
        }
        self.get_active_item()
    }
}
