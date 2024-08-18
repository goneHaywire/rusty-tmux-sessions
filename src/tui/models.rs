use std::io;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    style::Stylize,
    widgets::{Block, Widget},
    Frame,
};
use ratatui_macros::horizontal;

use crate::tmux::{
    sessions::{self, Session},
    windows::Window,
};

enum ScrollDirection {
    Up,
    Down,
    Top,
    Bottom,
}

enum SectionDirection {
    Left,
    Right,
}

use super::main::Tui;

#[derive(PartialEq)]
enum Section {
    Sessions,
    Windows,
}

pub struct App {
    selected_session_index: usize,
    selected_window_index: usize,
    show_hidden: bool,
    sessions: Vec<Session>,
    section: Section,
    is_renaming: bool,
    is_killing: bool,
    is_adding: bool,
    exit: bool,
}

impl Default for App {
    fn default() -> Self {
        App {
            selected_session_index: 0,
            selected_window_index: 0,
            show_hidden: true,
            sessions: vec![],
            section: Section::Sessions,
            is_renaming: false,
            is_killing: false,
            is_adding: false,
            exit: false,
        }
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut Buffer) {
        let [session_area, window_area] = horizontal![==50%, ==50%].areas(area);

        Block::bordered()
            //.title(" Sessions ".bold())
            .title(format!("{}", self.selected_session_index).bold())
            .render(session_area, buf);
        Block::bordered()
            //.title(" Windows ".bold())
            .title(format!("{}", self.selected_window_index).bold())
            .render(window_area, buf);
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut Tui) -> io::Result<()> {
        self.sessions = sessions::get_sessions().unwrap();
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_press) if key_press.kind == KeyEventKind::Press => {
                let keycode = key_press.code;

                match keycode {
                    KeyCode::Char('q') => self.exit(),
                    KeyCode::Char('j') if self.section == Section::Sessions => {
                        self.session_scroll(ScrollDirection::Down)
                    }
                    KeyCode::Char('k') if self.section == Section::Sessions => {
                        self.session_scroll(ScrollDirection::Up)
                    }
                    KeyCode::Char('j') if self.section == Section::Windows => {
                        self.window_scroll(ScrollDirection::Down)
                    }
                    KeyCode::Char('k') if self.section == Section::Windows => {
                        self.window_scroll(ScrollDirection::Up)
                    }
                    KeyCode::Char('g') if self.section == Section::Sessions => {
                        self.session_scroll(ScrollDirection::Top)
                    }
                    KeyCode::Char('G') if self.section == Section::Sessions => {
                        self.session_scroll(ScrollDirection::Bottom)
                    }
                    KeyCode::Char('g') if self.section == Section::Windows => {
                        self.window_scroll(ScrollDirection::Top)
                    }
                    KeyCode::Char('G') if self.section == Section::Windows => {
                        self.window_scroll(ScrollDirection::Bottom)
                    }
                    KeyCode::Char('h') => self.change_section(SectionDirection::Left),
                    KeyCode::Char('l') => self.change_section(SectionDirection::Right),
                    KeyCode::Char('H') => self.toggle_hidden(),
                    KeyCode::Char('a') => self.toggle_is_adding(),
                    KeyCode::Char('d') => self.toggle_is_killing(),
                    KeyCode::Char('c') => self.toggle_is_renaming(),
                    _ => {}
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn session_scroll(&mut self, dir: ScrollDirection) {
        let max_index = self.get_visible_sessions().len() - 1;

        self.selected_session_index = match (dir, self.selected_session_index) {
            (ScrollDirection::Top, _) => 0,
            (ScrollDirection::Bottom, _) => max_index,
            (ScrollDirection::Down, x) if x == max_index => 0,
            (ScrollDirection::Down, _) => self.selected_session_index + 1,
            (ScrollDirection::Up, 0) => max_index,
            (ScrollDirection::Up, _) => self.selected_session_index - 1,
        };
    }

    fn window_scroll(&mut self, dir: ScrollDirection) {
        let max_index: usize = self.get_windows().len() - 1;

        self.selected_window_index = match (dir, self.selected_window_index) {
            (ScrollDirection::Top, _) => 0,
            (ScrollDirection::Bottom, _) => max_index,
            (ScrollDirection::Down, x) if x == max_index => 0,
            (ScrollDirection::Down, _) => self.selected_window_index + 1,
            (ScrollDirection::Up, 0) => max_index,
            (ScrollDirection::Up, _) => self.selected_window_index - 1,
        };
    }

    fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
    }

    fn toggle_is_renaming(&mut self) {
        self.is_renaming = !self.is_renaming;
        todo!("handle action for both sections")
    }

    fn toggle_is_adding(&mut self) {
        self.is_adding = !self.is_adding;
        todo!("handle action for both sections")
    }

    fn toggle_is_killing(&mut self) {
        self.is_killing = !self.is_killing;
        todo!("handle action for both sections")
    }

    fn change_section(&mut self, direction: SectionDirection) {
        self.section = match direction {
            //SectionDirection::Left if self.active_section == Section::Sessions => Section::Sessions,
            SectionDirection::Left => {
                //self.selected_session_index = 0;
                Section::Sessions
            }
            //SectionDirection::Right if self.active_section == Section::Windows => Section::Windows,
            SectionDirection::Right => {
                //self.selected_window_index = 0;
                Section::Windows
            }
        };
    }

    pub fn get_visible_sessions(&self) -> Vec<Session> {
        match self.show_hidden {
            true => self.sessions.clone(),
            false => self
                .sessions
                .clone()
                .into_iter()
                .filter(|s| !s.is_hidden)
                .collect(),
        }
    }

    fn get_windows(&self) -> Vec<Window> {
        self.get_visible_sessions()[self.selected_session_index]
            .windows
            .clone()
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}
