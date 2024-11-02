use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{block::Title, Block, BorderType, List, Paragraph},
    Frame,
};
use ratatui_macros::{horizontal, vertical};

use crate::tui::mode::Section;

use super::{app::App, mode::Mode};

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

    let active_item: Option<String> = match app.mode {
        Select(Sessions) | Delete(Sessions) | Rename(Sessions, _) => {
            Some(app.session_list.get_active_item())
        }
        Select(Windows) | Delete(Windows) | Rename(Windows, _) => {
            Some(app.window_list.get_active_item())
        }
        _ => None,
    };
    let active_item = active_item.map(|name| Span::from(name).bold());

    let title = match &app.mode {
        Select(Sessions) => vec![
            " Session: ".into(),
            active_item.expect("should have a selected item").green(),
            " ".into(),
        ],
        Select(Windows) => vec![
            " Window: ".into(),
            active_item.expect("should have a selected item").green(),
            " ".into(),
        ],

        Create(Sessions, ..) => vec![" Enter new session name ".yellow()],
        Create(Windows, ..) => vec![" Enter new window name ".yellow()],

        Delete(Sessions) => vec![
            " Window: ".into(),
            active_item.expect("should have a selected item").red(),
            " ".into(),
        ],
        Delete(Windows) => vec![
            " Window: ".into(),
            active_item.expect("should have a selected item").red(),
            " ".into(),
        ],

        Rename(Sessions, _) => vec![
            " Enter new name for session ".into(),
            active_item.expect("should have a selected item").magenta(),
            " ".into(),
        ],
        Rename(Windows, _) => vec![
            " Enter new name for window ".into(),
            active_item.expect("should have a selected item").magenta(),
            " ".into(),
        ],
        _ => vec!["".into()],
    };
    let title = Title::from(Line::from(title));

    let text = match &app.mode {
        Select(_) => vec!["selecting".into()],

        Delete(Sessions) => vec![" Press y to delete session or any other key to cancel ".red()],
        Delete(Windows) => vec![" Press y to delete window or any other key to cancel ".red()],

        Rename(_, input) | Create(_, input, _) => vec![input.content.as_str().into()],
        _ => vec!["".into()],
    };
    let text = Text::from(Line::from(text));

    let block = Block::bordered()
        .border_type(BorderType::Thick)
        .title(title);
    let block = match app.mode {
        Delete(_) => block.border_style(Style::default().red()),
        Create(..) => block.border_style(Style::default().green()),
        _ => block,
    };

    frame.render_widget(Paragraph::new(text).block(block), area);
}

pub fn render_session_list(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::bordered()
        .border_type(BorderType::Thick)
        .title(" Sessions ".bold());

    let list: List = app.session_list.items.iter().map(|s| s as &str).collect();
    let list = list.highlight_symbol("> ").block(block);

    let mut state = app.session_list.state.clone();
    frame.render_stateful_widget(list, area, &mut state);
}

pub fn render_window_list(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::bordered()
        .border_type(BorderType::Thick)
        .title(" Windows ".bold());

    let list: List = app.window_list.items.iter().map(|w| w as &str).collect();
    let list = list.highlight_symbol("> ").block(block);

    let mut state = app.window_list.state.clone();
    frame.render_stateful_widget(list, area, &mut state);
}
