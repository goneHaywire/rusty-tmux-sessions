use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{
    cmp,
    collections::HashMap,
    io,
    sync::mpsc::{self, Receiver, Sender},
};

use crate::{
    tmux::{
        sessions::{Session, SessionService},
        tmux_command::WindowPos,
        windows::{Window, WindowService},
    },
    tui::{action::Actions, tmux_list::Selection, view},
};

use super::{
    event::Events,
    logger::Logger,
    mode::{Mode, Section, ToggleResult::*},
    tmux_list::StatefulList,
    tui::TUI,
};

pub struct App {
    // TODO: add the structs of other widgets
    // since the App controls state, it will modify the state of each widget
    // for ex. the scrolling of the lists will be handled by the structs of each list widget
    // the App will handle the fetching of the data from tmux and persisting it
    pub session_list: StatefulList,
    pub window_list: StatefulList,
    sessions: HashMap<String, Session>,
    windows: HashMap<String, Vec<Window>>,
    pub mode: Mode,
    atx: Sender<Actions<'static>>,
    arx: Receiver<Actions<'static>>,
}

impl App {
    fn load_sessions(&mut self) {
        self.sessions.clear();
        let sessions = SessionService::get_all().unwrap();

        for session in sessions {
            self.sessions.insert(session.name.clone(), session);
        }
    }

    fn load_windows(&mut self) {
        let session_name = self.session_list.get_active_item();
        let windows = WindowService::get_all(&session_name).unwrap();

        self.windows.insert(session_name.clone(), windows);
    }

    fn hydrate_session_list(&mut self, selection: Option<Selection>) {
        let names = self.sessions.keys().cloned().collect();
        self.session_list.items(names);
        if let Some(selection) = selection {
            self.session_list.select(selection);
        }
    }

    fn hydrate_window_list(&mut self, selection: Option<Selection>) {
        let session_name = self.session_list.get_active_item();
        let names = self
            .windows
            .get(&session_name)
            .unwrap_or_else(|| panic!("can't find windows for session {}", session_name))
            .iter()
            .map(|w| w.name.clone())
            .collect();
        self.window_list.items(names);
        if let Some(selection) = selection {
            self.window_list.select(selection);
        }
    }

    fn toggle_is_renaming(&mut self) {
        if let Toggled(mut mode) = self.mode.toggle_rename() {
            self.mode = match mode {
                Mode::Rename(Section::Sessions, ref mut input) => {
                    input.set_content(&self.session_list.get_active_item());
                    mode
                }
                Mode::Rename(Section::Windows, ref mut input) => {
                    input.set_content(&self.window_list.get_active_item());
                    mode
                }
                _ => mode,
            };
        }
    }

    fn attach_session(&mut self) {
        let current_session = self.session_list.get_active_item();
        if let Ok(mode) =
            SessionService::attach(&current_session).and_then(|_| self.mode.exit().into())
        {
            self.mode = mode;
        }
    }

    fn attach_window(&mut self) {
        let session = self.session_list.get_active_item();
        let window = self.window_list.get_active_item();
        let id = self.get_selected_window(&session).unwrap().id;

        if let Ok(mode) = WindowService::attach(&id).and_then(|_| self.mode.exit().into()) {
            self.mode = mode;
        }
    }

    fn rename_session(&mut self, new_name: &str) {
        let old_name = self.session_list.get_active_item();

        if SessionService::rename(&old_name, new_name).is_ok() {
            let sesh = self
                .sessions
                .remove(&old_name)
                .expect("session should be stored");
            self.sessions.insert(new_name.into(), sesh);
            self.hydrate_session_list(None);
        };
        self.toggle_is_renaming();
    }

    fn rename_window(&mut self, new_name: &str) {
        let session = self.session_list.get_active_item();
        let id = self.get_selected_window(&session).unwrap().id;

        if WindowService::rename(&id, new_name).is_ok() {
            if let Ok(window) = WindowService::get_window(&session, &id) {
                self.windows.entry(session).and_modify(|windows| {
                    if let Some(index) = windows.iter().position(|w| w.id == id) {
                        windows.push(window);
                        windows.swap_remove(index);
                    }
                });
                self.hydrate_window_list(None);
            }
        };
        self.toggle_is_renaming();
    }

    fn create_window(&mut self, name: &str, pos: WindowPos) {
        let curr_window_name = self.window_list.get_active_item();
        let session = self.session_list.get_active_item();
        let id = self.get_selected_window(&session).unwrap().id;
        Logger::log(&format!("creating window after win with id {id}"));

        if WindowService::create(name, &id, &pos).is_ok() {
            let window = WindowService::get_window(&session, &id).unwrap();
            Logger::log(&format!("got window after create {:?}", window));
            self.windows.entry(session).and_modify(|windows| {
                let current_window = windows.iter().position(|w| w.id == id).unwrap();
                let index = match pos {
                    WindowPos::Before => cmp::max(current_window - 1, 0),
                    WindowPos::After => cmp::min(current_window + 1, windows.len()),
                };
                windows.insert(index, window);
            });
            self.hydrate_window_list(None);
        }
        self.toggle_is_adding();
    }

    fn create_session(&mut self, name: &str) {
        // TODO: create will return the name of the new item
        if SessionService::create(name).is_ok() {
            let session = SessionService::get_session(name).unwrap();
            self.sessions.insert(session.name.clone(), session);
            self.hydrate_session_list(None);
        }
        self.toggle_is_adding();
    }

    fn kill_session(&mut self) {
        let session = self.session_list.get_active_item();
        if SessionService::kill(&session).is_ok() {
            self.sessions.remove(&session);
            self.windows.remove(&session);
            Logger::log(&format!("sessions: {:?}", self.sessions));
            Logger::log(&format!("windows: {:?}", self.windows));

            self.hydrate_session_list(None);
            self.atx
                .clone()
                .send(Actions::SelectSession(Selection::Prev))
                .unwrap();
            // self.set_visible_windows(None);
        }
        self.toggle_is_killing();
    }

    fn kill_window(&mut self) {
        let session = self.session_list.get_active_item();
        let id = self.get_selected_window(&session).unwrap().id;

        if WindowService::kill(&id).is_ok() {
            self.windows.entry(session.clone()).and_modify(|windows| {
                windows.retain(|w| w.id != id);
            });
            if self.windows.get(&session).unwrap().is_empty() {
                // self.sessions.remove(&session);
                // self.windows.remove(&session);
                self.kill_session();

                self.mode = self.mode.go_to_section(Section::Sessions);
                // self.update_session_list(Some(Selection::Prev));
                // self.set_visible_windows(None);
            } else {
                self.toggle_is_killing();
                self.hydrate_window_list(Some(Selection::PrevNoWrap));
            }
        }
    }

    fn is_duplicate_window(&self, session: &String, window: String) -> bool {
        self.windows
            .get(session)
            .unwrap()
            .iter()
            .any(|w| w.name == window)
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

    fn get_window_by_name(&self, session: &String, name: &String) -> Option<&Window> {
        self.windows
            .get(session)
            .unwrap()
            .iter()
            .find(|&w| w.name == *name)
    }

    fn get_selected_window(&self, session: &String) -> Option<&Window> {
        match self.window_list.state.selected() {
            Some(index) => self.windows.get(session).unwrap().get(index),
            None => None,
        }
    }
}

impl Default for App {
    fn default() -> Self {
        let (atx, arx) = mpsc::channel::<Actions>();
        Self {
            session_list: Default::default(),
            window_list: Default::default(),
            sessions: Default::default(),
            windows: Default::default(),
            mode: Default::default(),
            atx,
            arx,
        }
    }
}

impl App {
    pub fn run(&mut self, tui: &mut TUI) -> io::Result<()> {
        while !self.mode.should_exit() {
            if let Ok(action) = self.arx.try_recv() {
                self.handle_action(action);
            } else {
                let state = &self.mode.clone();
                let action = match tui.events.next() {
                    Events::Key(k) => App::handle_key_events(state, k),
                    Events::Resize(_, _) | Events::Tick => Actions::Tick,
                    Events::Init => Actions::Init,
                    Events::Quit => Actions::Quit,
                };
                self.handle_action(action);
            }

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

        match (key, mode) {
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
                self.load_sessions();
                self.hydrate_session_list(None);
                self.load_windows();
                self.hydrate_window_list(None);
            }
            Quit => self.exit(),
            LoadSessions => self.load_sessions(),
            LoadWindows => self.load_windows(),
            CreateSession(name) => self.create_session(name),
            CreateWindow(name) => self.create_window(name, WindowPos::After),
            SelectSession(selection) => {
                let session = self.session_list.select(selection);
                if !self.windows.contains_key(&session) {
                    self.load_windows();
                }
                self.hydrate_window_list(Some(Selection::First));
            }
            SelectWindow(selection) => {
                self.window_list.select(selection);
            }
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
