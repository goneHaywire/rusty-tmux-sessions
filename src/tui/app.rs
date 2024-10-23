use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{
    collections::HashMap,
    io,
    sync::{Arc, Mutex},
    thread::spawn,
};

use crate::{
    tmux::{
        sessions::{Session, SessionService},
        windows::{Window, WindowService},
    },
    tui::{action::Actions, tmux_list::Selection, view},
};

use super::{
    event::Events,
    mode::{Mode, Section, ToggleResult::*},
    tmux_list::StatefulList,
    tui::TUI,
};

#[derive(Default)]
pub struct App {
    // TODO: add the structs of other widgets
    // since the App controls state, it will modify the state of each widget
    // for ex. the scrolling of the lists will be handled by the structs of each list widget
    // the App will handle the fetching of the data from tmux and persisting it
    pub session_list: StatefulList,
    pub window_list: StatefulList,
    sessions: Arc<Mutex<HashMap<String, Session>>>,
    windows: Arc<Mutex<HashMap<String, Vec<Window>>>>,
    pub mode: Mode,
}

impl App {
    fn load_sessions_list(&mut self) {
        let clone = self.sessions.clone();

        spawn(move || {
            let mut sessions = clone.lock().unwrap();
            for sesh in SessionService::get_all().unwrap() {
                sessions.insert(sesh.name.clone(), sesh);
            }
        })
        .join()
        .expect("can't join from sessions thread");

        let sessions = self
            .sessions
            .clone()
            .lock()
            .unwrap()
            .values()
            .map(|s| s.name.clone())
            .collect();
        self.session_list = StatefulList::default().with_items(sessions);
    }

    fn load_window_list(&mut self) {
        let selected_session = self.session_list.get_active_item();
        let clone = self.windows.clone();

        spawn(move || {
            let mut windows = clone.lock().unwrap();
            windows.insert(
                selected_session.clone(),
                WindowService::get_all(&selected_session).unwrap(),
            )
        })
        .join()
        .expect("can't join from windows thread");

        let mutex = &self.windows.clone();
        let windows = mutex.lock().unwrap();
        let windows: Vec<String> = windows
            .get(&self.session_list.get_active_item())
            .unwrap()
            .clone()
            .into_iter()
            .map(|w| w.name)
            .collect();

        self.window_list = StatefulList::default().with_items(windows);
    }

    fn toggle_is_renaming(&mut self) {
        if let Toggled(mut mode) = self.mode.toggle_rename() {
            self.mode = match mode {
                Mode::Rename(Section::Sessions, ref mut _input) => {
                    _input.set_content(&self.session_list.get_active_item());
                    mode
                }
                Mode::Rename(Section::Windows, ref mut _input) => {
                    _input.set_content(&self.session_list.get_active_item());
                    mode
                }
                _ => mode,
            };
        }
    }

    fn attach_session(&mut self) {
        let current_session = self.session_list.get_active_item();
        SessionService::attach(&current_session);
        self.mode = self.mode.exit().unwrap();
    }

    fn attach_window(&mut self) {
        let session = self.session_list.get_active_item();
        let window = self.window_list.get_active_item();

        WindowService::attach(&session, &window);
        self.mode = self.mode.exit().unwrap();
    }

    fn rename_session(&mut self, new_name: &str) {
        let old_name = self.session_list.get_active_item();

        SessionService::rename(&old_name, new_name);
        self.load_sessions_list();
        self.toggle_is_renaming();
    }

    fn rename_window(&mut self, new_name: &str) {
        let session_name = self.session_list.get_active_item();
        let old_name = self.window_list.get_active_item();

        WindowService::rename(&session_name, &old_name, new_name);
        self.load_window_list();
        self.toggle_is_renaming();
    }

    fn create_window(&mut self, name: &str) {
        let curr_window_name = self.window_list.get_active_item().clone();
        let session = self.session_list.get_active_item();
        self.toggle_is_adding();

        WindowService::create(&session, &curr_window_name, name);
        self.load_window_list();
    }

    fn create_session(&mut self, name: &str) {
        self.toggle_is_adding();

        SessionService::create(name);
        self.load_sessions_list();
    }

    fn kill_session(&mut self) {
        let session = self.session_list.get_active_item();
        SessionService::kill(&session);
        self.toggle_is_killing();
        self.load_sessions_list();
    }

    fn kill_window(&mut self) {
        let session = self.session_list.get_active_item();
        let window = self.window_list.get_active_item();
        WindowService::kill(&session, &window);
        self.toggle_is_killing();
        if self.window_list.items.len() == 1 {
            self.load_sessions_list();
            self.mode = self.mode.go_to_section(Section::Sessions);
        }
        self.load_window_list();
    }

    fn input_key(&mut self, key: KeyCode) {
        match &mut self.mode {
            Mode::Create(_, ref mut input) => input.handle_key(key),
            Mode::Rename(_, ref mut input) => input.handle_key(key),
            _ => {}
        };
    }

    fn cancel_input(&mut self) {
        match &mut self.mode {
            Mode::Create(_, ref mut input) => input.clear(),
            Mode::Rename(_, ref mut input) => input.clear(),
            _ => {}
        }
    }

    fn toggle_is_adding(&mut self) {
        self.mode = self.mode.toggle_create().unwrap();
    }

    fn toggle_is_killing(&mut self) {
        self.mode = self.mode.toggle_delete().unwrap();
    }

    fn exit(&mut self) {
        self.mode = self.mode.exit().unwrap();
    }
}

impl App {
    pub fn run(&mut self, tui: &mut TUI) -> io::Result<()> {
        while !self.mode.should_exit() {
            let state = self.mode.clone();
            let action = match tui.events.next() {
                Events::Key(k) => App::handle_key_events(&state, k),
                Events::Resize(_, _) | Events::Tick => Actions::Tick,
                Events::Init => Actions::Init,
                Events::Quit => Actions::Quit,
            };
            // TODO: in the future you can create a dedicated action channel to dispatch actions directly instead of only waiting for events to trigger them
            self.handle_action(action);

            // draw the screen
            // TODO: decide where to interface with the view
            tui.terminal.draw(|frame| view::render(frame, self))?;
        }
        Ok(())
    }

    fn handle_key_events(mode: &Mode, key: KeyEvent) -> Actions {
        use KeyCode::Char;
        use Mode::*;
        use Section::*;

        match (key, &mode) {
            // renaming & creating
            (
                KeyEvent {
                    code: Char(' '), ..
                },
                Rename(Sessions, input),
            ) => Actions::RenameSession(&input.content),
            (
                KeyEvent {
                    code: Char(' '), ..
                },
                Rename(Windows, input),
            ) => Actions::RenameWindow(&input.content),
            (
                KeyEvent {
                    code: Char(' '), ..
                },
                Create(Sessions, input),
            ) => Actions::CreateSession(&input.content),
            (
                KeyEvent {
                    code: Char(' '), ..
                },
                Create(Windows, input),
            ) => Actions::CreateWindow(&input.content),
            (
                KeyEvent {
                    code: KeyCode::Esc, ..
                },
                Create(..),
            ) => Actions::ToggleCreate,
            (
                KeyEvent {
                    code: KeyCode::Esc, ..
                },
                Rename(..),
            ) => Actions::ToggleRename,
            (
                KeyEvent {
                    code: Char('w'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                },
                Rename(..) | Create(..),
            ) => Actions::ClearInput,
            (KeyEvent { code: key, .. }, Rename(..) | Create(..)) => Actions::InputKey(key),

            // deletion handlers
            (
                KeyEvent {
                    code: Char('d'), ..
                },
                _,
            ) => Actions::ToggleDelete,
            (
                KeyEvent {
                    code: Char('y'), ..
                },
                Delete(Sessions),
            ) => Actions::KillSession,
            (
                KeyEvent {
                    code: Char('y'), ..
                },
                Delete(Windows),
            ) => Actions::KillWindow,
            (_, Delete(_)) => Actions::ToggleDelete,

            // selection handlers for sessions
            (
                KeyEvent {
                    code: Char('j'), ..
                },
                Select(Sessions),
            ) => Actions::SelectSession(Selection::Next),
            (
                KeyEvent {
                    code: Char('k'), ..
                },
                Select(Sessions),
            ) => Actions::SelectSession(Selection::Prev),
            (
                KeyEvent {
                    code: Char('g'), ..
                },
                Select(Sessions),
            ) => Actions::SelectSession(Selection::First),
            (
                KeyEvent {
                    code: Char('G'), ..
                },
                Select(Sessions),
            ) => Actions::SelectSession(Selection::Last),
            (
                KeyEvent {
                    code: Char('l'), ..
                },
                Select(Sessions),
            ) => Actions::ChangeSection(Windows),
            (
                KeyEvent {
                    code: Char('H'), ..
                },
                Select(Sessions),
            ) => Actions::ToggleHidden,
            (
                KeyEvent {
                    code: Char(' '), ..
                }
                | KeyEvent {
                    code: KeyCode::Enter,
                    ..
                },
                Select(Sessions),
            ) => Actions::AttachSession,

            // selection handlers for windows
            (
                KeyEvent {
                    code: Char('j'), ..
                },
                Select(Windows),
            ) => Actions::SelectWindow(Selection::Next),
            (
                KeyEvent {
                    code: Char('k'), ..
                },
                Select(Windows),
            ) => Actions::SelectWindow(Selection::Prev),
            (
                KeyEvent {
                    code: Char('g'), ..
                },
                Select(Windows),
            ) => Actions::SelectWindow(Selection::First),
            (
                KeyEvent {
                    code: Char('G'), ..
                },
                Select(Windows),
            ) => Actions::SelectWindow(Selection::Last),
            (
                KeyEvent {
                    code: Char('h'), ..
                },
                Select(Windows),
            ) => Actions::ChangeSection(Sessions),
            (
                KeyEvent {
                    code: Char(' '), ..
                }
                | KeyEvent {
                    code: KeyCode::Enter,
                    ..
                },
                Select(Windows),
            ) => Actions::AttachWindow,

            (
                KeyEvent {
                    code: Char('a'), ..
                },
                Select(_),
            ) => Actions::ToggleCreate,
            (
                KeyEvent {
                    code: Char('c'), ..
                },
                Select(_),
            ) => Actions::ToggleRename,

            (
                KeyEvent {
                    code: Char('q'), ..
                }
                | KeyEvent {
                    code: KeyCode::Esc, ..
                },
                _,
            ) => Actions::Quit,
            _ => Actions::Tick,
        }
    }

    fn handle_action(&mut self, action: Actions) {
        use Actions::*;

        match action {
            Tick => {}
            Init => {
                self.load_sessions_list();
                self.load_window_list();
            }
            Quit => self.exit(),
            LoadSessions => self.load_sessions_list(),
            LoadWindows => self.load_window_list(),
            CreateSession(name) => self.create_session(name),
            CreateWindow(name) => self.create_window(name),
            SelectSession(selection) => {
                self.session_list.select(selection);
                self.load_window_list();
            }
            SelectWindow(selection) => self.window_list.select(selection),
            KillSession => self.kill_session(),
            KillWindow => self.kill_window(),
            RenameSession(name) => self.rename_session(name),
            RenameWindow(name) => self.rename_window(name),
            ToggleHelp => todo!(),
            ChangeSection(section) => self.mode = self.mode.go_to_section(section),
            ClearInput => self.cancel_input(),
            InputKey(key) => self.input_key(key),
            ToggleCreate => self.toggle_is_adding(),
            ToggleRename => self.toggle_is_renaming(),
            ToggleDelete => self.toggle_is_killing(),
            ToggleHidden => todo!(),
            AttachSession => self.attach_session(),
            AttachWindow => self.attach_window(),
        };
    }
}
