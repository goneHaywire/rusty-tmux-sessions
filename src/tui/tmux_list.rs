use std::ops::Index;

use ratatui::widgets::ListState;

#[derive(PartialEq, Copy, Clone, Default)]
pub enum Selection {
    Index(Option<usize>),
    NextNoWrap,
    PrevNoWrap,
    Next,
    Prev,
    #[default]
    First,
    Last,
    Noop,
}

/// Wrapper for a stateful TUI list
///
/// * `items`: Vector of Sessions or Windows
/// * `state`: ListState
#[derive(Debug, Default)]
pub struct StatefulList {
    pub items: Vec<String>,
    pub state: ListState,
}

impl StatefulList {
    /// replace the list items and select the last item if the previous selection exceeded the new
    /// items length
    ///
    /// * `items`: new list to replace the old
    pub fn set_items(&mut self, items: Vec<String>) {
        self.items = items;
        match self.state.selected() {
            Some(index) => {
                if index >= self.items.len() {
                    self.select(Selection::Last);
                }
            }
            None if !self.items.is_empty() => {
                self.select(Selection::First);
            }
            None => (),
        }
    }

    //pub fn get_active_item(&self) -> &String {
    //    let active_idx = self
    //        .state
    //        .selected()
    //        .expect("there should always be a selection");
    //    self.items.index(active_idx)
    //}

    pub fn get_active_item(&self) -> Option<&String> {
        self.state.selected().map(|idx| self.items.index(idx))
    }

    /// selection function that handles multiple selection options and returns the selected &item
    ///
    /// * `selection`: Selection
    pub fn select(&mut self, selection: Selection) -> Option<&String> {
        use Selection::*;

        if self.items.is_empty() {
            return None;
        };
        let last_index = self.items.len() - 1;
        let current = self.state.selected().unwrap();

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
