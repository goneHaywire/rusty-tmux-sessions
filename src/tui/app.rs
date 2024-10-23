use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::{
    collections::HashMap,
    io,
    sync::{Arc, Mutex},
    thread::spawn,
};

use crate::tmux::{
    sessions::{Session, SessionService},
    windows::{Window, WindowService},
};

use super::{
    input::InputState,
    mode::Mode,
    tmux_list::{Selection, StatefulList},
};

#[derive(PartialEq, Default)]
pub enum Section {
    #[default]
    Sessions,
    Windows,
}

#[derive(Default)]
pub struct App {
    // TODO: add the structs of other widgets
    // since the App controls state, it will modify the state of each widget
    // for ex. the scrolling of the lists will be handled by the structs of each list widget
    // the App will handle the fetching of the data from tmux and persisting it
    pub session_list: StatefulList<Session>,
    pub window_list: StatefulList<Window>,
    sessions: Arc<Mutex<HashMap<String, Session>>>,
    windows: Arc<Mutex<HashMap<String, Window>>>,
    pub section: Section,
    pub mode: Mode,
    pub input: InputState,
}

impl App {
    // pub fn run(&mut self, terminal: &mut Tui) -> io::Result<()> {
    //     self.load_sessions_list();
    //     self.load_window_list();
    //
    //     while !self.state.should_exit() {
    //         terminal.draw(|frame| {
    //             view::render(frame, self);
    //             // frame.render_stateful_widget(AppWidget, frame.area(), self);
    //         })?;
    //         self.handle_events()?;
    //     }
    //     Ok(())
    // }

    fn handle_events(&mut self) -> io::Result<()> {
        use KeyCode::Char;
        use Section::*;
        use Selection::*;

        match event::read()? {
            Event::Key(key_press) if key_press.kind == KeyEventKind::Press => {
                let keycode = key_press.code;

                match (keycode, &self.section) {
                    // renaming handlers
                    (Char(' '), Sessions) if self.mode.is_renaming() => self.rename_session(),
                    (Char(' '), Windows) if self.mode.is_renaming() => self.rename_window(),

                    // creating handlers
                    (Char(' '), Sessions) if self.mode.is_adding() => self.create_session(),
                    (Char(' '), Windows) if self.mode.is_adding() => self.create_window(),

                    // renaming & creating
                    (KeyCode::Esc, _) if self.mode.is_renaming() || self.mode.is_adding() => {
                        self.cancel_input()
                    }
                    (key, _) if self.mode.is_renaming() || self.mode.is_adding() => {
                        self.input.handle_key(key)
                    }

                    // deletion handlers
                    (Char('d'), _) => self.toggle_is_killing(),
                    (Char('y'), Sessions) if self.mode.is_killing() => self.kill_session(),
                    (Char('y'), Windows) if self.mode.is_killing() => self.kill_window(),
                    _ if self.mode.is_killing() => self.toggle_is_killing(),

                    // selection handlers for sessions
                    (Char('j'), Sessions) => {
                        self.session_list.select(Next);
                        self.load_window_list();
                    }
                    (Char('k'), Sessions) => {
                        self.session_list.select(Prev);
                        self.load_window_list();
                    }
                    (Char('g'), Sessions) => {
                        self.session_list.select(First);
                        self.load_window_list();
                    }
                    (Char('G'), Sessions) => {
                        self.session_list.select(Last);
                        self.load_window_list();
                    }
                    (Char('l'), Sessions) => self.go_to_section(Windows),
                    (Char('H'), Sessions) => self.session_list.toggle_hidden(),
                    (Char(' ') | KeyCode::Enter, Sessions) => self.attach_session(),

                    // selection handlers for windows
                    (Char('j'), Windows) => self.window_list.select(Next),
                    (Char('k'), Windows) => self.window_list.select(Prev),
                    (Char('g'), Windows) => self.window_list.select(First),
                    (Char('G'), Windows) => self.window_list.select(Last),
                    (Char('h'), Windows) => self.go_to_section(Sessions),
                    (Char(' ') | KeyCode::Enter, Windows) => self.attach_window(),

                    (Char('a'), _) => self.toggle_is_adding(),
                    (Char('c'), _) => self.toggle_is_renaming(),

                    (Char('q') | KeyCode::Esc, _) => self.exit(),
                    _ => (),
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn load_sessions_list(&mut self) {
        let sessions = Arc::new(Mutex::new(vec![]));
        let clone = sessions.clone();

        spawn(move || {
            let mut sessions = clone.lock().unwrap();
            *sessions = SessionService::get_all().unwrap();
        })
        .join()
        .expect("can't join from sessions thread");

        let sessions = Arc::try_unwrap(sessions).unwrap().into_inner().unwrap();
        self.session_list = StatefulList::default().with_items(sessions);
    }

    fn load_window_list(&mut self) {
        let selected_session = self.session_list.get_active_item().name;
        let windows = Arc::new(Mutex::new(vec![]));
        let clone = windows.clone();

        spawn(move || {
            let mut windows = clone.lock().unwrap();
            *windows = WindowService::get_all(&selected_session).unwrap();
        })
        .join()
        .expect("can't join from windows thread");

        let windows = Arc::try_unwrap(windows).unwrap().into_inner().unwrap();
        self.window_list = StatefulList::default().with_items(windows);
    }

    fn toggle_is_renaming(&mut self) {
        self.mode = self.mode.toggle_renaming();
        let active_name = match &self.section {
            Section::Sessions => self.session_list.get_active_item().name,
            Section::Windows => self.window_list.get_active_item().name,
        };
        self.input = InputState::new(&active_name);
    }

    fn attach_session(&mut self) {
        let current_session = self.session_list.get_active_item().name;
        SessionService::attach(&current_session);
        self.mode = self.mode.exit();
    }

    fn attach_window(&mut self) {
        let session = self.session_list.get_active_item().name;
        let window = self.window_list.get_active_item().name;

        WindowService::attach(&session, &window);
        self.mode = self.mode.exit();
    }

    fn rename_session(&mut self) {
        let new_name = self.input.submit();
        let old_name = self.session_list.get_active_item().name;

        SessionService::rename(&old_name, &new_name);
        self.load_sessions_list();
        self.toggle_is_renaming();
    }

    fn rename_window(&mut self) {
        let session_name = self.session_list.get_active_item().name;
        let new_name = self.input.submit();
        let old_name = self.window_list.get_active_item().name;

        WindowService::rename(&session_name, &old_name, &new_name);
        self.load_window_list();
        self.toggle_is_renaming();
    }

    fn create_window(&mut self) {
        let name = self.input.submit();
        let curr_window_name = self.window_list.get_active_item().name.clone();
        let session = self.session_list.get_active_item().name;
        self.toggle_is_adding();

        WindowService::create(&session, &curr_window_name, &name);
        self.load_window_list();
    }

    fn create_session(&mut self) {
        let name = self.input.submit();
        self.toggle_is_adding();

        SessionService::create(&name);
        self.load_sessions_list();
    }

    fn kill_session(&mut self) {
        let session = self.session_list.get_active_item().name;
        SessionService::kill(&session);
        self.toggle_is_killing();
        self.load_sessions_list();
    }

    fn kill_window(&mut self) {
        let session = self.session_list.get_active_item().name;
        let window = self.window_list.get_active_item().name;
        WindowService::kill(&session, &window);
        self.toggle_is_killing();
        if self.window_list.items.len() == 1 {
            self.load_sessions_list();
            self.go_to_section(Section::Sessions);
        }
        self.load_window_list();
    }

    fn cancel_input(&mut self) {
        match self.mode {
            Mode::Create => self.toggle_is_adding(),
            Mode::Rename => self.toggle_is_renaming(),
            _ => {}
        }
        self.input.reset();
    }

    fn toggle_is_adding(&mut self) {
        self.mode = self.mode.toggle_create();
        self.input = InputState::default();
    }

    fn toggle_is_killing(&mut self) {
        self.mode = self.mode.toggle_delete();
    }

    fn go_to_section(&mut self, section: Section) {
        self.section = section;
    }

    fn exit(&mut self) {
        self.mode = self.mode.exit();
    }
}
