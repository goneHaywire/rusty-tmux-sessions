use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{block::Title, Block, BorderType, List, Paragraph},
    Frame,
};
use ratatui_macros::{horizontal, vertical};

use super::{app::App, mode::Mode};
use crate::tui::app::Section;

pub fn render(frame: &mut Frame, app: &mut App) {
    let [body, footer_area] = vertical![*=1, ==3].areas(frame.area());
    let [session_area, window_area] = horizontal![==50%, ==50%].areas(body);

    render_session_list(frame, session_area, app);
    render_window_list(frame, window_area, app);
    render_footer(frame, footer_area, app);
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    use Mode::*;
    use Section::*;

    let active_item = match app.section {
        Sessions => app.session_list.get_active_item().name,
        Windows => app.window_list.get_active_item().name,
    };
    let active_item = active_item.as_str().bold();

    let title = match (&app.mode, &app.section) {
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

    let text = match (&app.mode, &app.section) {
        (Selecting, Sessions) => vec!["selecting".into()],
        (Selecting, Windows) => vec!["selecting".into()],

        (Deleting, Sessions) => {
            vec![" Press y to delete session or any other key to cancel ".red()]
        }
        (Deleting, Windows) => {
            vec![" Press y to delete window or any other key to cancel ".red()]
        }

        (Renaming | Creating, _) => vec![app.input.content.as_str().into()],
        _ => vec!["".into()],
    };
    let text = Text::from(Line::from(text));

    let block = Block::bordered()
        .border_type(BorderType::Thick)
        .title(title);
    let block = match app.mode {
        Deleting => block.border_style(Style::default().red()),
        Creating => block.border_style(Style::default().green()),
        _ => block,
    };

    frame.render_widget(Paragraph::new(text).block(block), area);
}

pub fn render_session_list(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::bordered()
        .border_type(BorderType::Thick)
        .title(" Sessions ".bold());

    let list: List = app
        .session_list
        .items
        .iter()
        .map(|s| &s.name as &str)
        .collect();
    let list = list.highlight_symbol("> ").block(block);

    let mut state = app.session_list.state.clone();
    frame.render_stateful_widget(list, area, &mut state);
}

pub fn render_window_list(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::bordered()
        .border_type(BorderType::Thick)
        .title(" Windows ".bold());

    let list: List = app
        .window_list
        .items
        .iter()
        .map(|w| &w.name as &str)
        .collect();
    let list = list.highlight_symbol("> ").block(block);

    let mut state = app.window_list.state.clone();
    frame.render_stateful_widget(list, area, &mut state);
}
