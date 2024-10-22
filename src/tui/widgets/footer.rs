use std::sync::mpsc::SyncSender;

use ratatui::{style::Stylize, widgets::StatefulWidget};

use crate::tui::action::Action;

pub struct Footer {
    tx: SyncSender<Action>,
}

impl Footer {
    fn new(tx: SyncSender<Action>) -> Self {
        Self { tx }
    }
}

struct FooterWidget;

impl StatefulWidget for Footer {
    type State = Footer;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
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
}
