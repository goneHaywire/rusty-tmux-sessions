use ratatui::crossterm::event::KeyCode;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct InputState {
    pub content: String,
    index: usize,
}

impl InputState {
    pub fn set_content(&mut self, content: &str) {
        self.content = content.into();
    }

    pub fn clear(&mut self) {
        self.index = 0;
        self.content.clear();
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Backspace => self.remove_char(),
            KeyCode::Char(c) if c.is_ascii() => self.add_char(c),
            _ => {}
        };
    }

    fn move_cursor_left(&mut self) {
        self.index = self.index.saturating_sub(1);
    }

    fn move_cursor_right(&mut self) {
        let new_index = self.index.saturating_add(1);
        self.index = new_index.clamp(0, self.content.chars().count());
    }

    fn add_char(&mut self, char: char) {
        if char.is_ascii() {
            self.content.push(char);
            self.move_cursor_right();
        }
    }

    fn remove_char(&mut self) {
        let _ = self.content.pop();
        self.move_cursor_left();
    }
}
