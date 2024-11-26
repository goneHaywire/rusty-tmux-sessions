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
        windows::{IdW, Window, WindowService},
    },
    tui::{action::Actions as A, tmux_list::Selection, view},
};

use super::{
    event::Events,
    mode::{CommandKind, Mode, Section, ToggleResult::*},
    tmux_list::StatefulList,
    tui::TUI,
};

pub struct App {
    pub session_list: StatefulList,
    pub window_list: StatefulList,
    pub sessions: HashMap<String, Session>,
    windows: HashMap<String, Vec<Window>>,
    pub mode: Mode,
    atx: Sender<A<'static>>,
    arx: Receiver<A<'static>>,
    pub show_hidden: bool,
}

impl App {
    fn load_sessions(&mut self) {
        self.sessions.clear();
        let sessions = SessionService::get_all().unwrap_or_default();

        for session in sessions {
            self.sessions.insert(session.name.clone(), session);
        }
    }

    fn load_windows(&mut self) {
        if let Some(session) = self.session_list.get_active_item() {
            let windows = WindowService::get_all(session).unwrap();
            self.windows.insert(session.clone(), windows);
        }
    }

    fn update_window(&mut self, id: &IdW) {
        if let Ok(window) = WindowService::get_window(id) {
            self.windows
                .entry(window.session_name.clone())
                .and_modify(|windows| {
                    if let Some(index) = windows.iter().position(|w| w.id == *id) {
                        windows.push(window);
                        windows.swap_remove(index);
                    }
                });
            self.hydrate_window_list();
        }
    }

    pub fn get_active_session(&self) -> Option<&Session> {
        self.session_list
            .get_active_item()
            .and_then(|s| self.sessions.get(s))
    }

    pub fn get_active_window(&self) -> Option<&Window> {
        self.session_list
            .get_active_item()
            .and_then(|session| self.get_window(session))
            .and_then(|(window, session)| {
                self.windows
                    .get(session)
                    .unwrap()
                    .iter()
                    .find(|w| w.id == window.id)
            })
    }

    fn hydrate_session_list(&mut self) {
        let mut sessions: Vec<Session> = self.sessions.values().cloned().collect();
        if !sessions.is_empty() {
            sessions.sort_by_key(|s| s.last_attached);
            sessions.reverse();
            sessions.rotate_left(1);
            if !self.show_hidden {
                sessions.retain(|s| !s.is_hidden);
            }
        }
        let sessions = sessions.into_iter().map(|s| s.name).collect();
        self.session_list.set_items(sessions);
    }

    fn hydrate_window_list(&mut self) {
        if let Some(session_name) = self.session_list.get_active_item() {
            let names = self
                .windows
                .get(session_name)
                .unwrap_or_else(|| panic!("can't find windows for session {}", session_name))
                .iter()
                .map(|w| w.name.clone())
                .collect();
            self.window_list.set_items(names);
        }
    }

    fn attach_session(&mut self) {
        if let Some(current_session) = self.session_list.get_active_item() {
            if let Ok(mode) =
                SessionService::attach(current_session).and_then(|_| self.mode.exit().into())
            {
                self.mode = mode;
            }
        }
    }

    fn attach_window(&mut self) {
        if let Some((window, _)) = self
            .session_list
            .get_active_item()
            .and_then(|session| self.get_window(session))
        {
            if let Ok(mode) =
                WindowService::attach(&window.id).and_then(|_| self.mode.exit().into())
            {
                self.mode = mode;
            }
        }
    }

    fn rename_session(&mut self, new_name: &str) {
        self.atx.send(A::ExitRename).unwrap();
        if let Some(old_name) = self.session_list.get_active_item() {
            if SessionService::rename(old_name, new_name).is_ok() {
                if let Ok(session) = SessionService::get_session(new_name) {
                    self.sessions
                        .remove(old_name)
                        .expect("session should be stored");
                    self.sessions.insert(new_name.into(), session);

                    let sesh = self
                        .windows
                        .remove(old_name)
                        .expect("session should have windows");
                    self.windows.insert(new_name.into(), sesh);
                    self.hydrate_session_list();
                }
            }
        };
    }

    fn rename_window(&mut self, new_name: &str) {
        if let Some((window, _)) = self
            .session_list
            .get_active_item()
            .and_then(|session| self.get_window(session))
        {
            self.atx.send(A::ExitRename).unwrap();

            let _ = WindowService::rename(&window.id, new_name)
                .map(|_| self.atx.send(A::UpdateWindow(window.id)));
        }
    }

    fn create_window(&mut self, name: &str, pos: Option<WindowPos>) {
        self.atx.send(A::ExitCreate).unwrap();
        if let Some((window, session)) = self
            .session_list
            .get_active_item()
            .and_then(|session| self.get_window(session))
        {
            let pos = pos.unwrap_or_default();

            if WindowService::create(name, &window.id, &pos).is_ok() {
                let window = WindowService::get_last_created_window_id(session)
                    .and_then(|id| WindowService::get_window(&id))
                    .unwrap();

                self.windows.entry(session.clone()).and_modify(|windows| {
                    let current_window = windows.iter().position(|w| w.id == window.id).unwrap();
                    let index = match pos {
                        WindowPos::Before => current_window,
                        WindowPos::After => cmp::min(current_window + 1, windows.len()),
                    };
                    windows.insert(index, window);
                });
                let action = match pos {
                    WindowPos::Before => A::Select(Section::Windows, Selection::Noop),
                    WindowPos::After => A::Select(Section::Windows, Selection::NextNoWrap),
                };
                self.atx.send(action).unwrap();
            }
        }
    }

    fn create_session(&mut self, name: &str) {
        self.atx.send(A::ExitCreate).unwrap();
        if SessionService::create(name).is_ok() {
            let session = SessionService::get_session(name).unwrap();
            self.sessions.insert(session.name.clone(), session);

            // TODO: consider switching to the created sessions
            self.atx
                .send(A::Select(Section::Sessions, Selection::NextNoWrap))
                .unwrap();
        }
    }

    fn kill_session(&mut self) {
        self.atx.send(A::ExitDelete).unwrap();
        if let Some(session) = self.session_list.get_active_item() {
            if SessionService::kill(session).is_ok() {
                self.atx.send(A::RemoveSession(session.clone())).unwrap();
                self.atx
                    .send(A::Select(Section::Sessions, Selection::PrevNoWrap))
                    .unwrap();
            }
        }
    }

    fn remove_session(&mut self, session: &String) {
        self.sessions.remove(session);
        self.windows.remove(session);
    }

    fn remove_window(&mut self, session: String, id: &IdW) {
        self.windows.entry(session.clone()).and_modify(|windows| {
            windows.retain(|w| w.id != *id);
        });
    }

    fn kill_window(&mut self) {
        if let Some((window, session)) = self
            .session_list
            .get_active_item()
            .and_then(|session| self.get_window(session))
        {
            self.atx.send(A::ExitDelete).unwrap();

            if self.windows.get(session).unwrap().len() == 1 {
                self.atx.send(A::EnterDelete).unwrap();
                self.atx.send(A::ChangeSection(Section::Sessions)).unwrap();
                self.atx.send(A::Kill(Section::Sessions)).unwrap();
                return;
            }
            if WindowService::kill(&window.id).is_ok() {
                self.atx
                    .send(A::RemoveWindow(session.clone(), window.id))
                    .unwrap();
                self.atx
                    .send(A::Select(Section::Windows, Selection::PrevNoWrap))
                    .unwrap();
            }
        }
    }

    fn toggle_hide_session(&self) {
        if let Some(session) = self
            .session_list
            .get_active_item()
            .and_then(|session| self.sessions.get(session))
        {
            let action = match session.is_hidden {
                true => A::ShowSession,
                false => A::HideSession,
            };
            self.atx.send(action).unwrap();
        }
    }

    fn hide_session(&mut self) {
        if let Some(session) = self
            .session_list
            .get_active_item()
            .and_then(|session| SessionService::hide(session).ok().map(|_| session))
            .and_then(|session| self.sessions.get_mut(session))
        {
            session.is_hidden = true;
            self.atx
                .send(A::Select(Section::Sessions, Selection::Noop))
                .unwrap();
        };
    }

    //if let Some(session) = self.sessions.get_mut(&session) {
    //    if session.is_attached {
    //        return;
    //    }
    //
    //    if SessionService::hide(&session.name).is_ok() {
    //        session.is_hidden = true;
    //        self.atx
    //            .send(A::Select(Section::Sessions, Selection::Noop))
    //            .unwrap();
    //    };
    //};

    fn show_session(&mut self) {
        if let Some(session) = self
            .session_list
            .get_active_item()
            .and_then(|session| SessionService::show(session).ok().map(|_| session))
            .and_then(|session| self.sessions.get_mut(session))
        {
            session.is_hidden = false;
            self.atx
                .send(A::Select(Section::Sessions, Selection::Noop))
                .unwrap();
        };
    }

    //if let Some(session) = self.sessions.get_mut(&session) {
    //    if session.is_attached {
    //        return;
    //    }
    //
    //    if SessionService::show(&session.name).is_ok() {
    //        session.is_hidden = false;
    //        self.atx
    //            .send(A::Select(Section::Sessions, Selection::Noop))
    //            .unwrap();
    //    };
    //};

    fn send_command(&mut self, kind: CommandKind, keys: String) {
        self.atx.send(A::ExitSendCommand).unwrap();
        // TODO: since the event loop runs continuously without delay, the new running command is not correctly updated,
        // if this persists after the migration to tokio, add a manual sleep for a few millis
        self.session_list
            .get_active_item()
            .and_then(|session| self.get_window(session))
            .and_then(|(window, _)| {
                WindowService::send_keys(&window.id, keys.as_bytes(), kind)
                    .ok()
                    .map(|_| window)
            })
            .inspect(|window| {
                let _ = self.atx.send(A::UpdateWindow(window.id));
            });
    }

    //fn send_command(&mut self, kind: CommandKind, keys: String) {
    //    self.atx.send(A::ExitSendCommand).unwrap();
    //
    //    if let Some((window, _)) = self
    //        .session_list
    //        .get_active_item()
    //        .and_then(|session| self.get_window(session))
    //    {
    //        let _ = WindowService::send_keys(&window.id, keys.as_bytes(), kind)
    //            .map(|_| self.atx.send(A::UpdateWindow(window.id)));
    //    }
    //
    //    // TODO: since the event loop runs continuously without delay, the new running command is not correctly updated,
    //    // if this persists after the migration to tokio, add a manual sleep for a few millis
    //}

    fn input_key(&mut self, key: KeyCode) {
        match &mut self.mode {
            Mode::Create(_, ref mut input, _)
            | Mode::Rename(_, ref mut input)
            | Mode::SendCommand(_, ref mut input) => input.handle_key(key),
            _ => {}
        };
    }

    fn cancel_input(&mut self) {
        match &mut self.mode {
            Mode::Create(_, ref mut input, _) => input.clear(),
            Mode::Rename(_, ref mut input) => input.clear(),
            _ => {}
        }
    }

    //fn get_selected_window(&self, session: &String) -> Option<&Window> {
    //    match self.window_list.state.selected() {
    //        Some(index) => self.windows.get(session).unwrap().get(index),
    //        None => None,
    //    }
    //}

    fn get_window<'a, 'b>(&'a self, session: &'b String) -> Option<(&'a Window, &'b String)> {
        let window = self.window_list.state.selected().and_then(|idx| {
            self.windows
                .get(session)
                .and_then(|windows| windows.get(idx))
        });
        window.zip(Some(session))
    }

    fn enter_rename(&mut self) {
        if let Toggled(mut mode) = self.mode.enter_rename() {
            self.mode = match mode {
                Mode::Rename(Section::Sessions, ref mut input) => {
                    self.session_list
                        .get_active_item()
                        .inspect(|session| input.set_content(session));
                    mode
                }
                Mode::Rename(Section::Windows, ref mut input) => {
                    self.window_list
                        .get_active_item()
                        .inspect(|window| input.set_content(window));
                    mode
                }
                _ => mode,
            };
        }
    }

    fn enter_create(&mut self, pos: Option<WindowPos>) {
        self.mode = self.mode.enter_create(pos).unwrap();
    }

    fn enter_delete(&mut self) {
        self.mode = self.mode.enter_delete().unwrap();
    }

    fn exit_create(&mut self) {
        self.mode = self.mode.exit_create().unwrap();
    }

    fn exit_rename(&mut self) {
        self.mode = self.mode.exit_rename().unwrap();
    }

    fn exit_delete(&mut self) {
        self.mode = self.mode.exit_delete().unwrap();
    }

    fn exit(&mut self) {
        self.mode = self.mode.exit().unwrap();
    }

    fn enter_send_command(&mut self) {
        self.mode = self.mode.enter_send_command().unwrap();
    }

    fn exit_send_command(&mut self) {
        self.mode = self.mode.exit_send_command().unwrap();
    }

    fn change_command_kind(&mut self) {
        self.mode = self.mode.change_command_mode().unwrap();
    }

    fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
        self.atx
            .send(A::Select(Section::Sessions, Selection::Noop))
            .unwrap();
    }
}

impl Default for App {
    fn default() -> Self {
        let (atx, arx) = mpsc::channel::<A>();
        Self {
            session_list: Default::default(),
            window_list: Default::default(),
            sessions: Default::default(),
            windows: Default::default(),
            mode: Default::default(),
            atx,
            arx,
            show_hidden: false,
        }
    }
}

impl App {
    pub fn run(&mut self, tui: &mut TUI) -> io::Result<()> {
        while !self.mode.should_exit() {
            while let Ok(action) = self.arx.try_recv() {
                self.handle_action(action);
            }
            let state = &self.mode.clone();
            let action = match tui.events.next() {
                Events::Key(k) => App::handle_key_events(state, self.show_hidden, k),
                Events::Resize(_, _) | Events::Tick => A::Tick,
                Events::Init => A::Init,
                Events::Quit => A::Quit,
            };
            self.handle_action(action);

            // draw the screen
            // TODO: decide where to interface with the view
            tui.terminal.draw(|frame| view::render(frame, self))?;
        }
        Ok(())
    }

    fn handle_key_events(mode: &Mode, show_hidden: bool, key: KeyEvent) -> A {
        use KeyCode::Char;
        use Mode::*;
        use Section::*;

        match (key, mode) {
            // renaming & creating
            (
                KeyEvent {
                    code: Char(' '), ..
                },
                Rename(section, input),
            ) => A::Rename(*section, &input.content),
            (
                KeyEvent {
                    code: Char(' '), ..
                },
                Create(section, input, pos),
            ) => A::Create(*section, &input.content, *pos),
            (
                KeyEvent {
                    code: KeyCode::Enter,
                    ..
                },
                SendCommand(k, input),
            ) => A::SendCommand(*k, input.content.clone()),
            (
                KeyEvent {
                    code: KeyCode::Tab, ..
                },
                SendCommand(..),
            ) => A::ChangeCommandKind,
            (
                KeyEvent {
                    code: KeyCode::Esc, ..
                },
                Create(..),
            ) => A::ExitCreate,
            (
                KeyEvent {
                    code: KeyCode::Esc, ..
                },
                Rename(..),
            ) => A::ExitRename,
            (
                KeyEvent {
                    code: KeyCode::Esc, ..
                },
                SendCommand(..),
            ) => A::ExitSendCommand,
            (
                KeyEvent {
                    code: Char('w'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                },
                Rename(..) | Create(..) | SendCommand(..),
            ) => A::ClearInput,
            (KeyEvent { code: key, .. }, Rename(..) | Create(..) | SendCommand(..)) => {
                A::InputKey(key)
            }

            // deletion handlers
            (
                KeyEvent {
                    code: Char('d'), ..
                },
                _,
            ) => A::EnterDelete,
            (
                KeyEvent {
                    code: Char('y'), ..
                },
                Delete(section),
            ) => A::Kill(*section),
            (_, Delete(_)) => A::ExitDelete,

            // selection handlers for sessions
            (
                KeyEvent {
                    code: Char('j'), ..
                },
                Select(section),
            ) => A::Select(*section, Selection::Next),
            (
                KeyEvent {
                    code: Char('k'), ..
                },
                Select(section),
            ) => A::Select(*section, Selection::Prev),
            (
                KeyEvent {
                    code: Char('g'), ..
                },
                Select(section),
            ) => A::Select(*section, Selection::First),
            (
                KeyEvent {
                    code: Char('G'), ..
                },
                Select(section),
            ) => A::Select(*section, Selection::Last),
            (
                KeyEvent {
                    code: Char('l'), ..
                },
                Select(Sessions),
            ) => A::ChangeSection(Windows),
            (
                KeyEvent {
                    code: Char('H'), ..
                },
                Select(Sessions),
            ) => A::ToggleHidden,
            (
                KeyEvent {
                    code: Char('z'), ..
                },
                Select(Sessions),
            ) if show_hidden => A::ToggleHideSession,
            (
                KeyEvent {
                    code: Char('z'), ..
                },
                Select(Sessions),
            ) => A::HideSession,
            (
                KeyEvent {
                    code: Char(' '), ..
                }
                | KeyEvent {
                    code: KeyCode::Enter,
                    ..
                },
                Select(Sessions),
            ) => A::AttachSession,

            // selection handlers for windows
            (
                KeyEvent {
                    code: Char('h'), ..
                },
                Select(Windows),
            ) => A::ChangeSection(Sessions),
            (
                KeyEvent {
                    code: Char(' '), ..
                }
                | KeyEvent {
                    code: KeyCode::Enter,
                    ..
                },
                Select(Windows),
            ) => A::AttachWindow,

            (
                KeyEvent {
                    code: Char('o'), ..
                },
                Select(Sessions),
            ) => A::EnterCreate(None),
            (
                KeyEvent {
                    code: Char('o'), ..
                },
                Select(Windows),
            ) => A::EnterCreate(Some(WindowPos::After)),
            (
                KeyEvent {
                    code: Char('O'), ..
                },
                Select(Windows),
            ) => A::EnterCreate(Some(WindowPos::Before)),
            (
                KeyEvent {
                    code: Char('c'), ..
                },
                Select(_),
            ) => A::EnterRename,
            (
                KeyEvent {
                    code: Char('s'), ..
                },
                Select(Windows),
            ) => A::EnterSendCommand,

            (
                KeyEvent {
                    code: Char('q'), ..
                }
                | KeyEvent {
                    code: KeyCode::Esc, ..
                },
                _,
            ) => A::Quit,
            _ => A::Tick,
        }
    }

    fn handle_action(&mut self, action: A) {
        use A::*;

        match action {
            Tick => {}
            Init => {
                self.load_sessions();
                self.hydrate_session_list();
                self.load_windows();
                self.hydrate_window_list();
            }
            Quit => self.exit(),
            LoadSessions => self.load_sessions(),
            LoadWindows => self.load_windows(),
            HydrateSessions => self.hydrate_session_list(),
            HydrateWindows => self.hydrate_window_list(),
            UpdateWindow(id) => self.update_window(&id),
            Create(Section::Sessions, name, _) => self.create_session(name),
            Create(Section::Windows, name, pos) => self.create_window(name, pos),
            Select(Section::Sessions, selection) => {
                if self.sessions.len() > 1 {
                    self.hydrate_session_list();
                    let session = self.session_list.select(selection);
                    if !self.windows.contains_key(session.unwrap()) {
                        self.load_windows();
                    }
                    self.hydrate_window_list();
                    if selection != Selection::Noop {
                        self.window_list.select(Selection::Index(Some(0)));
                    }
                }
            }
            Select(Section::Windows, selection) => {
                self.hydrate_window_list();
                self.window_list.select(selection);
            }
            Kill(Section::Sessions) => self.kill_session(),
            Kill(Section::Windows) => self.kill_window(),
            RemoveSession(session) => self.remove_session(&session),
            RemoveWindow(window, id) => self.remove_window(window, &id),
            Rename(Section::Sessions, name) => self.rename_session(name),
            Rename(Section::Windows, name) => self.rename_window(name),
            SendCommand(kind, keys) => self.send_command(kind, keys),
            ToggleHideSession => self.toggle_hide_session(),
            HideSession => self.hide_session(),
            ShowSession => self.show_session(),
            ToggleHelp => todo!(),
            ChangeSection(section) => self.mode = self.mode.change_section(section),
            ClearInput => self.cancel_input(),
            InputKey(key) => self.input_key(key),
            EnterCreate(pos) => self.enter_create(pos),
            EnterRename => self.enter_rename(),
            EnterDelete => self.enter_delete(),
            EnterSendCommand => self.enter_send_command(),
            ChangeCommandKind => self.change_command_kind(),
            ExitCreate => self.exit_create(),
            ExitRename => self.exit_rename(),
            ExitDelete => self.exit_delete(),
            ExitSendCommand => self.exit_send_command(),
            ToggleHidden => self.toggle_hidden(),
            AttachSession => self.attach_session(),
            AttachWindow => self.attach_window(),
        };
    }
}
