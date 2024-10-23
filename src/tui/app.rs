use ratatui::crossterm::event::{KeyCode, KeyEvent};
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
    tui::{action::Actions, view},
};

use super::{
    event::Events,
    input::InputState,
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
    pub session_list: StatefulList<Session>,
    pub window_list: StatefulList<Window>,
    sessions: Arc<Mutex<HashMap<String, Session>>>,
    windows: Arc<Mutex<HashMap<String, Window>>>,
    // pub section: Section,
    pub mode: Mode,
    pub input: InputState,
}

impl App {
    pub fn run(&mut self, tui: &mut TUI) -> io::Result<()> {
        while !self.mode.should_exit() {
            println!("got event! {:?}", tui.events.next());
            let state = self.mode.clone();
            let action = match tui.events.next() {
                Events::Key(k) => App::handle_key_events(&state, k),
                Events::Resize(_, _) | Events::Tick => Actions::Tick,
                Events::Init => Actions::Init,
                Events::Quit => Actions::Quit,
            };
            // TODO: in the future you can create a dedicated action channel to dispatch actions directly instead of only waiting for events to trigger them
            self.handle_action(action);
        }

        // draw the screen
        // TODO: decide where to interface with the view
        tui.terminal.draw(|frame| view::render(frame, self))?;

        // while !self.state.should_exit() {
        //     terminal.draw(|frame| {
        //         view::render(frame, self);
        //         // frame.render_stateful_widget(AppWidget, frame.area(), self);
        //     })?;
        //     self.handle_events()?;
        // }
        Ok(())
    }

    fn handle_key_events(mode: &Mode, key: KeyEvent) -> Actions {
        use KeyCode::Char;
        use Mode::*;
        use Section::*;

        match (key.code, &mode) {
            (Char(' '), Rename(Sessions, input)) => Actions::RenameSession(&input.content),
            (Char(' '), Rename(Windows, input)) => Actions::RenameWindow(&input.content),

            // creating handlers
            (Char(' '), Create(Sessions, input)) => Actions::CreateSession(&input.content),
            (Char(' '), Create(Windows, input)) => Actions::CreateWindow(&input.content),

            // renaming & creating
            // TODO: think about this
            (KeyCode::Esc, Create(..)) => Actions::ToggleCreate,
            (KeyCode::Esc, Rename(..)) => Actions::ToggleRename,
            // (key, Rename(_) | Create(_)) => Actions::InputKey(key),

            // deletion handlers
            (Char('d'), _) => Actions::ToggleDelete,
            (Char('y'), Delete(Sessions)) => Actions::KillSession,
            (Char('y'), Delete(Windows)) => Actions::KillWindow,
            (_, Delete(_)) => Actions::ToggleDelete,
            _ => Actions::Tick,
        }
    }

    // fn handle_events(&mut self, event: Events) -> io::Result<()> {
    //     use KeyCode::Char;
    //     use Selection::*;
    //
    //     match event::read()? {
    //         Event::Key(key_press) if key_press.kind == KeyEventKind::Press => {
    //             let keycode = key_press.code;
    //
    //             match (keycode, &self.mode) {
    //                 // renaming handlers
    //                 (Char(' '), Sessions) if self.mode.is_renaming() => self.rename_session(),
    //                 (Char(' '), Windows) if self.mode.is_renaming() => self.rename_window(),
    //
    //                 // creating handlers
    //                 (Char(' '), Sessions) if self.mode.is_adding() => self.create_session(),
    //                 (Char(' '), Windows) if self.mode.is_adding() => self.create_window(),
    //
    //                 // renaming & creating
    //                 (KeyCode::Esc, _) if self.mode.is_renaming() || self.mode.is_adding() => {
    //                     self.cancel_input()
    //                 }
    //                 (key, _) if self.mode.is_renaming() || self.mode.is_adding() => {
    //                     self.input.handle_key(key)
    //                 }
    //
    //                 // deletion handlers
    //                 (Char('d'), _) => self.toggle_is_killing(),
    //                 (Char('y'), Sessions) if self.mode.is_killing() => self.kill_session(),
    //                 (Char('y'), Windows) if self.mode.is_killing() => self.kill_window(),
    //                 _ if self.mode.is_killing() => self.toggle_is_killing(),
    //
    //                 // selection handlers for sessions
    //                 (Char('j'), Sessions) => {
    //                     self.session_list.select(Next);
    //                     self.load_window_list();
    //                 }
    //                 (Char('k'), Sessions) => {
    //                     self.session_list.select(Prev);
    //                     self.load_window_list();
    //                 }
    //                 (Char('g'), Sessions) => {
    //                     self.session_list.select(First);
    //                     self.load_window_list();
    //                 }
    //                 (Char('G'), Sessions) => {
    //                     self.session_list.select(Last);
    //                     self.load_window_list();
    //                 }
    //                 (Char('l'), Sessions) => self.go_to_section(Windows),
    //                 (Char('H'), Sessions) => self.session_list.toggle_hidden(),
    //                 (Char(' ') | KeyCode::Enter, Sessions) => self.attach_session(),
    //
    //                 // selection handlers for windows
    //                 (Char('j'), Windows) => self.window_list.select(Next),
    //                 (Char('k'), Windows) => self.window_list.select(Prev),
    //                 (Char('g'), Windows) => self.window_list.select(First),
    //                 (Char('G'), Windows) => self.window_list.select(Last),
    //                 (Char('h'), Windows) => self.go_to_section(Sessions),
    //                 (Char(' ') | KeyCode::Enter, Windows) => self.attach_window(),
    //
    //                 (Char('a'), _) => self.toggle_is_adding(),
    //                 (Char('c'), _) => self.toggle_is_renaming(),
    //
    //                 (Char('q') | KeyCode::Esc, _) => self.exit(),
    //                 _ => (),
    //             }
    //         }
    //         _ => {}
    //     }
    //
    //     Ok(())
    // }

    fn handle_action(&mut self, action: Actions) {
        use Actions::*;

        match action {
            Tick => {}
            Init => {
                self.load_sessions_list();
                self.load_window_list();
            }
            Quit => {}
            LoadSessions => self.load_sessions_list(),
            LoadWindows => self.load_window_list(),
            CreateSession(name) => self.create_session(name),
            CreateWindow(name) => self.create_window(name),
            SelectSession(_) => todo!(),
            SelectWindow(_) => todo!(),
            KillSession => self.kill_session(),
            KillWindow => self.kill_window(),
            RenameSession(name) => self.rename_session(name),
            RenameWindow(name) => self.rename_window(name),
            ToggleHelp => todo!(),
            ChangeSection(_) => todo!(),
            CancelInput => todo!(),
            InputKey(_) => todo!(),
            ToggleCreate => todo!(),
            ToggleRename => todo!(),
            ToggleDelete => todo!(),
        };
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
        if let Toggled(mut mode) = self.mode.toggle_rename() {
            self.mode = match mode {
                Mode::Rename(Section::Sessions, ref mut _input) => {
                    _input.content(&self.session_list.get_active_item().name);
                    mode
                }
                Mode::Rename(Section::Windows, ref mut _input) => {
                    _input.content(&self.session_list.get_active_item().name);
                    mode
                }
                // Mode::Select(_) => self.cancel_input(),
                _ => mode,
            };
        }
    }

    fn attach_session(&mut self) {
        let current_session = self.session_list.get_active_item().name;
        SessionService::attach(&current_session);
        self.mode = self.mode.exit().unwrap();
    }

    fn attach_window(&mut self) {
        let session = self.session_list.get_active_item().name;
        let window = self.window_list.get_active_item().name;

        WindowService::attach(&session, &window);
        self.mode = self.mode.exit().unwrap();
    }

    fn rename_session(&mut self, new_name: &str) {
        let old_name = self.session_list.get_active_item().name;

        SessionService::rename(&old_name, &new_name);
        self.load_sessions_list();
        self.toggle_is_renaming();
    }

    fn rename_window(&mut self, new_name: &str) {
        let session_name = self.session_list.get_active_item().name;
        let old_name = self.window_list.get_active_item().name;

        WindowService::rename(&session_name, &old_name, &new_name);
        self.load_window_list();
        self.toggle_is_renaming();
    }

    fn create_window(&mut self, name: &str) {
        let curr_window_name = self.window_list.get_active_item().name.clone();
        let session = self.session_list.get_active_item().name;
        self.toggle_is_adding();

        WindowService::create(&session, &curr_window_name, &name);
        self.load_window_list();
    }

    fn create_session(&mut self, name: &str) {
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
            self.mode = self.mode.go_to_section(Section::Sessions);
        }
        self.load_window_list();
    }

    // fn cancel_input(&mut self) {
    //     match self.mode {
    //         Mode::Create(_, _) => self.toggle_is_adding(),
    //         Mode::Rename(_, _) => self.toggle_is_renaming(),
    //         _ => {}
    //     }
    //     self.input.reset();
    // }

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
