use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{block::Title, Block, BorderType, List, Paragraph, StatefulWidget, Widget},
    Frame,
};
use ratatui_macros::{horizontal, vertical};
use std::{
    io,
    sync::{Arc, Mutex},
    thread::spawn,
};

use crate::tmux::{
    sessions::{Session, SessionService},
    windows::{Window, WindowService},
};

use super::{
    app_state::AppState,
    input::InputState,
    main::Tui,
    tmux_list::{Selection, StatefulList},
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
    input: InputState,
}

impl Widget for &mut App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut Buffer) {
        let [body, footer_area] = vertical![*=1, ==3].areas(area);
        let [session_area, window_area] = horizontal![==50%, ==50%].areas(body);

        self.render_session_list(session_area, buf);
        self.render_window_list(window_area, buf);
        self.render_footer(footer_area, buf);
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut Tui) -> io::Result<()> {
        self.load_sessions_list();
        self.load_window_list();

        while !self.state.should_exit() {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        use KeyCode::Char;
        use Section::*;
        use Selection::*;

        match event::read()? {
            Event::Key(key_press) if key_press.kind == KeyEventKind::Press => {
                let keycode = key_press.code;

                match (keycode, &self.section) {
                    (Char('q'), _) => self.exit(),

                    // renaming handlers
                    (Char(' '), Sessions) if self.state.is_renaming() => self.rename_session(),
                    (Char(' '), Windows) if self.state.is_renaming() => self.rename_window(),

                    // creating handlers
                    (Char(' '), Sessions) if self.state.is_adding() => self.create_session(),
                    (Char(' '), Windows) if self.state.is_adding() => self.create_window(),

                    // renaming & creating
                    (KeyCode::Esc, _) if self.state.is_renaming() || self.state.is_adding() => {
                        self.cancel_input()
                    }
                    (key, _) if self.state.is_renaming() || self.state.is_adding() => {
                        self.input.handle_key(key)
                    }

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
                    (Char(' '), Sessions) => self.attach_session(),

                    (Char('j'), Windows) => self.window_list.select(Next),
                    (Char('k'), Windows) => self.window_list.select(Prev),
                    (Char('g'), Windows) => self.window_list.select(First),
                    (Char('G'), Windows) => self.window_list.select(Last),
                    (Char('h'), Windows) => self.go_to_section(Sessions),
                    (Char(' '), Windows) => self.attach_window(),

                    (Char('a'), _) => self.toggle_is_adding(),
                    (Char('c'), _) => self.toggle_is_renaming(),
                    _ => (),
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn render_session_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .border_type(BorderType::Thick)
            .title(" Sessions ".bold());

        let list: List = self
            .session_list
            .items
            .iter()
            .map(|s| &s.name as &str)
            .collect();
        let list = list.highlight_symbol("> ").block(block);

        StatefulWidget::render(list, area, buf, &mut self.session_list.state);
    }

    fn render_window_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .border_type(BorderType::Thick)
            .title(" Windows ".bold());

        let list: List = self
            .window_list
            .items
            .iter()
            .map(|w| &w.name as &str)
            .collect();
        let list = list.highlight_symbol("> ").block(block);

        StatefulWidget::render(list, area, buf, &mut self.window_list.state);
    }

    fn render_footer(&self, footer_area: Rect, buf: &mut Buffer) {
        use AppState::*;
        use Section::*;

        let active_item = match self.section {
            Sessions => self.session_list.get_active_item().name,
            Windows => self.window_list.get_active_item().name,
        };
        let active_item = active_item.as_str().bold();

        let title = match (&self.state, &self.section) {
            (Selecting, Sessions) => vec![" Session: ".into(), active_item.green(), " ".into()],
            (Selecting, Windows) => vec![" Window: ".into(), active_item.green(), " ".into()],

            (Creating, Sessions) => vec![" Enter new session name ".yellow()],
            (Creating, Windows) => vec![" Enter new window name ".yellow()],

            (Deleting, Sessions) => vec![" Window: ".into(), active_item.red(), " ".into()],
            (Deleting, Windows) => vec![" Window: ".into(), active_item.red(), " ".into()],

            (Renaming, Sessions) => vec![
                " Enter new name for session ".into(),
                active_item.magenta(),
                " ".into(),
            ],
            (Renaming, Windows) => vec![
                " Enter new name for window ".into(),
                active_item.magenta(),
                " ".into(),
            ],
            _ => vec!["".into()],
        };
        let title = Title::from(Line::from(title));

        let text = match (&self.state, &self.section) {
            (Selecting, Sessions) => vec!["selecting".into()],
            (Selecting, Windows) => vec!["selecting".into()],

            (Deleting, Sessions) => {
                vec![" Press y to delete session or any other key to cancel ".red()]
            }
            (Deleting, Windows) => {
                vec![" Press y to delete window or any other key to cancel ".red()]
            }

            (Renaming | Creating, _) => vec![self.input.content.as_str().into()],
            _ => vec!["".into()],
        };
        let text = Text::from(Line::from(text));

        let block = Block::bordered()
            .border_type(BorderType::Thick)
            .title(title);
        let block = match self.state {
            Deleting => block.border_style(Style::default().red()),
            Creating => block.border_style(Style::default().green()),
            _ => block,
        };

        Paragraph::new(text).block(block).render(footer_area, buf);
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

    fn render_frame(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn toggle_is_renaming(&mut self) {
        self.state = self.state.toggle_renaming();
        let active_name = match &self.section {
            Section::Sessions => self.session_list.get_active_item().name,
            Section::Windows => self.window_list.get_active_item().name,
        };
        self.input = InputState::new(&active_name);
    }

    fn attach_session(&mut self) {
        let current_session = self.session_list.get_active_item().name;
        let _ = SessionService::attach(&current_session);
        self.state = self.state.exit();
    }

    fn attach_window(&mut self) {
        let session = self.session_list.get_active_item().name;
        let window = self.window_list.get_active_item().name;
        let _ = SessionService::attach(&session);
        let _ = WindowService::attach(&window);
        self.state = self.state.exit();
    }

    fn rename_session(&mut self) {
        let new_name = self.input.submit();
        let old_name = self.session_list.get_active_item().name;
        let _ = SessionService::rename(&old_name, &new_name);
        self.load_sessions_list();
        self.toggle_is_renaming();
    }

    fn rename_window(&mut self) {
        let new_name = self.input.submit();
        let old_name = self.window_list.get_active_item().name;
        //dbg!(&new_name, &old_name);
        let _ = WindowService::rename(&old_name, &new_name);
        self.load_window_list();
        self.toggle_is_renaming();
    }

    fn create_window(&mut self) {
        let name = self.input.submit();
        let curr_window_name = self.window_list.get_active_item().name.clone();
        self.toggle_is_adding();

        WindowService::create(&curr_window_name, &name);
    }

    fn create_session(&mut self) {
        let name = self.input.submit();
        self.toggle_is_adding();

        SessionService::create(&name);
        self.load_sessions_list();
    }

    fn cancel_input(&mut self) {
        match self.state {
            AppState::Creating => self.toggle_is_adding(),
            AppState::Renaming => self.toggle_is_renaming(),
            _ => {}
        }
        self.input.reset();
    }

    fn toggle_is_adding(&mut self) {
        self.state = self.state.toggle_creating();
        self.input = InputState::default();
    }

    fn toggle_is_killing(&mut self) {
        self.state = self.state.toggle_deleting();
    }

    fn go_to_section(&mut self, section: Section) {
        self.section = section;
    }

    fn exit(&mut self) {
        self.state = self.state.exit();
    }
}
