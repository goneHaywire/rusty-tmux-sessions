use ratatui::{
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{block::Title, Block, BorderType, List, Padding, Paragraph},
    Frame,
};
use ratatui_macros::{horizontal, vertical};
use time_humanize::HumanTime;

use crate::tui::mode::Section;

use super::{app::App, mode::Mode};

pub fn render(frame: &mut Frame, app: &mut App) {
    let [body, footer_area] = vertical![*=1, <=3].areas(frame.area());
    let [session_area, window_area] = horizontal![==50%, ==50%].areas(body);

    render_session_list(frame, session_area, app);
    render_window_list(frame, window_area, app);
    render_footer(frame, footer_area, app);
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    use Mode::*;
    use Section::*;

    let block = Block::bordered()
        .border_type(BorderType::Thick)
        .padding(Padding::horizontal(1));

    let block = match app.mode {
        Delete(_) => block.border_style(Style::default().red()),
        Create(..) => block.border_style(Style::default().green()),
        _ => block,
    };

    let active_item = match app.mode {
        Select(Sessions) | Delete(Sessions) | Rename(Sessions, _) => {
            Some(app.session_list.get_active_item())
        }
        Select(Windows) | Delete(Windows) | Rename(Windows, _) | SendCommand(_) => {
            Some(app.window_list.get_active_item())
        }
        _ => None,
    }
    .map(|name| Span::from(name).bold());

    let title = match &app.mode {
        Select(Sessions) => vec![
            " Session ".into(),
            active_item.expect("should have a selected item").green(),
            " ".into(),
        ],
        Select(Windows) => {
            let window = app.get_active_window().unwrap();
            vec![
                " Window ".into(),
                active_item.expect("should have a selected item").green(),
                " (".yellow().bold(),
                window.panes_count.to_string().yellow().bold(),
                ") ".yellow().bold(),
            ]
        }

        Create(Sessions, ..) => vec![" Enter new session name ".yellow()],
        Create(Windows, ..) => vec![" Enter new window name ".yellow()],

        Delete(Sessions) => vec![
            " Session ".into(),
            active_item.expect("should have a selected item").red(),
            " ".into(),
        ],
        Delete(Windows) => vec![
            " Window ".into(),
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
        SendCommand(_) => vec![
            " Send command to window ".into(),
            active_item.expect("window should be selected").magenta(),
            " ".into(),
        ],
        _ => vec!["".into()],
    };
    let title = Title::from(Line::from(title));
    let block = block.title(title);

    let right_title = match app.mode {
        Select(Sessions) | Delete(Sessions) | Rename(Sessions, _) => {
            let session = app.get_active_session().unwrap();
            Some(
                session
                    .last_attached
                    .map(humanize_time)
                    .unwrap_or("never".into()),
            )
        }
        Select(Windows) | Delete(Windows) | Rename(Windows, _) | SendCommand(_) => {
            let window = app.get_active_window().unwrap();
            Some(humanize_time(window.last_active))
        }
        _ => None,
    }
    .map(|right_title| match app.mode {
        Select(..) | Rename(..) | SendCommand(..) => Line::from(vec![
            " active ".into(),
            right_title.bold().green(),
            " ".into(),
        ]),
        Delete(..) => {
            Line::from(vec![" active ".into(), right_title.bold(), " ".into()]).light_red()
        }
        _ => Line::from(right_title),
    });

    let block = if let Some(right_title) = right_title {
        block.title(Title::from(right_title).alignment(Alignment::Right))
    } else {
        block
    };

    let content = match app.mode {
        Select(Sessions) => {
            let session = app.get_active_session().unwrap();
            vec![
                "Created ".into(),
                humanize_time(session.created_at).green().bold(),
            ]
        }
        Select(Windows) => {
            let window = app.get_active_window().unwrap();
            vec![
                "Current command: ".into(),
                window
                    .current_command
                    .clone()
                    .unwrap_or("none".into())
                    .magenta()
                    .bold(),
            ]
        }
        Delete(Sessions) => {
            vec!["Press y to delete session or any other key to cancel".light_red()]
        }
        Delete(Windows) => vec!["Press y to delete window or any other key to cancel".red()],

        Rename(_, ref input) | Create(_, ref input, _) | SendCommand(ref input) => {
            vec![input.content.clone().into()]
        }
        _ => vec!["".into()],
    };
    let content = Paragraph::new(Text::from(Line::from(content)));

    frame.render_widget(content.block(block), area);
}

pub fn render_session_list(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::bordered().border_type(BorderType::Thick);
    let block = match app.session_list.items.len() {
        0 => block.title(" No sessions ".bold()),
        len => block.title(format!(" Sessions ({len}) ").bold()),
    };

    let list: List = app.session_list.items.iter().map(|s| s as &str).collect();
    let list = list.highlight_symbol("> ").block(block);

    let mut state = app.session_list.state.clone();
    frame.render_stateful_widget(list, area, &mut state);
}

pub fn render_window_list(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::bordered().border_type(BorderType::Thick);
    let block = match app.window_list.items.len() {
        0 => block.title(" No windows ".bold()),
        len => block.title(format!(" Windows ({len}) ").bold()),
    };

    let list: List = app.window_list.items.iter().map(|w| w as &str).collect();
    let list = list.highlight_symbol("> ").block(block);

    let mut state = app.window_list.state.clone();
    frame.render_stateful_widget(list, area, &mut state);
}

fn humanize_time(timestamp: u64) -> String {
    let time = HumanTime::from_duration_since_timestamp(timestamp).to_string();
    match time.as_str() {
        now @ "now" => {
            let mut s = "just ".to_string();
            s.push_str(now);
            s
        }
        _ => time,
    }
}
