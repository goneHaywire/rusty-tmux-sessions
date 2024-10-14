use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::Rect,
    style::Stylize,
    widgets::{Block, List, StatefulWidget, Widget},
    Frame,
};
use ratatui_macros::horizontal;
use std::io;

use crate::tmux::{
    sessions::{Session, SessionService},
    windows::{Window, WindowService},
};

use super::{
    app_state::AppState,
    main::Tui,
    tmux_list::{ScrollDirection, StatefulList},
};

#[derive(PartialEq, Default)]
enum Section {
    #[default]
    Sessions,
    Windows,
}

#[derive(Default)]
pub struct App {
    session_list: StatefulList<Session>,
    window_list: StatefulList<Window>,
    section: Section,
    state: AppState,
}

impl Widget for &mut App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut Buffer) {
        let [session_area, window_area] = horizontal![==50%, ==50%].areas(area);

        self.render_session_list(session_area, buf);
        self.render_window_list(window_area, buf);
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut Tui) -> io::Result<()> {
        self.load_sessions_list();
        self.load_window_list();

        while !self.state.is_quitting() {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn handle_events(&mut self) -> io::Result<()> {
        use KeyCode::Char;
        use ScrollDirection::*;
        use Section::*;

        match event::read()? {
            Event::Key(key_press) if key_press.kind == KeyEventKind::Press => {
                let keycode = key_press.code;

                match (keycode, &self.section) {
                    (Char('q'), _) => self.exit(),
                    (Char('d'), _) => self.toggle_is_killing(),
                    (Char('y'), _) if self.state.is_killing() => {
                        let curr_sesh = self.session_list.get_active_item();
                        let _ = SessionService::kill(&curr_sesh.name);
                        self.load_sessions_list();
                        self.load_window_list();
                        self.toggle_is_killing()
                    }
                    _ if self.state.is_killing() => self.toggle_is_killing(),

                    (Char('j'), Sessions) => {
                        self.session_list.scroll(Next);
                        self.load_window_list();
                    }
                    (Char('k'), Sessions) => {
                        self.session_list.scroll(Prev);
                        self.load_window_list();
                    }
                    (Char('g'), Sessions) => {
                        self.session_list.scroll(First);
                        self.load_window_list();
                    }
                    (Char('G'), Sessions) => {
                        self.session_list.scroll(Last);
                        self.load_window_list();
                    }
                    (Char('h'), Sessions) => (),
                    (Char('l'), Sessions) => self.go_to_section(Windows),
                    (Char('H'), Sessions) => self.session_list.toggle_hidden(),

                    (Char('j'), Windows) => self.window_list.scroll(Next),
                    (Char('k'), Windows) => self.window_list.scroll(Prev),
                    (Char('g'), Windows) => self.window_list.scroll(First),
                    (Char('G'), Windows) => self.window_list.scroll(Last),
                    (Char('h'), Windows) => self.go_to_section(Sessions),
                    (Char('l'), Windows) => (),

                    (Char('a'), _) => self.toggle_is_adding(),
                    (Char('c'), _) => self.toggle_is_renaming(),
                    _ => (),
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub fn render_session_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(" Sessions ".bold())
            .title(format!("{}", self.session_list.state.selected().unwrap()).bold());

        let list: List = self
            .session_list
            .items
            .iter()
            .map(|s| &s.name as &str)
            .collect();
        let list = list.highlight_symbol("> ").block(block);

        StatefulWidget::render(list, area, buf, &mut self.session_list.state);
    }

    pub fn render_window_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(" Windows ".bold())
            .title(format!("{}", self.window_list.state.selected().unwrap()).bold());

        let list: List = self
            .window_list
            .items
            .iter()
            .map(|w| &w.name as &str)
            .collect();
        let list = list.highlight_symbol("> ").block(block);

        StatefulWidget::render(list, area, buf, &mut self.window_list.state);
    }

    fn load_sessions_list(&mut self) {
        let sessions = SessionService::get_all().expect("error getting sessions");
        self.session_list = StatefulList::default().with_items(sessions);
    }

    fn load_window_list(&mut self) {
        let selected_session = &self.session_list.get_active_item().name;
        let windows = WindowService::get_all(selected_session).expect("error getting windows");

        self.window_list = StatefulList::default().with_items(windows);
    }

    fn render_frame(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn toggle_is_renaming(&mut self) {
        self.state = self.state.toggle_renaming();
        todo!("handle action for both sections")
    }

    fn toggle_is_adding(&mut self) {
        self.state = self.state.toggle_creating();

        SessionService::create("newsesh").unwrap();
        self.load_sessions_list();
        self.load_window_list();
        todo!("handle action for both sections")
    }

    fn toggle_is_killing(&mut self) {
        self.state = self.state.toggle_deleting();
    }

    fn go_to_section(&mut self, section: Section) {
        self.section = section;
    }

    fn exit(&mut self) {
        self.state = self.state.quit();
    }
}
