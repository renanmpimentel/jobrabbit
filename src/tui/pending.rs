//! Pending screen: actions requiring human intervention (captcha, fields, login).
//! Interactive: navigate (↑↓/jk), open URL (o/Enter), resolve (space/r).

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use super::{App, EditField};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    // Input box when answering a screening question.
    let answering = app.editing.as_ref() == Some(&EditField::AnswerInput);
    let chunks = if answering {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0)])
            .split(area)
    };
    let area = chunks[0];

    let items: Vec<ListItem> = if app.pending.is_empty() {
        vec![ListItem::new(Line::styled(
            "No pending actions. 🎉",
            Style::default().fg(Color::Green),
        ))]
    } else {
        app.pending
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let selected = i == app.pending_sel;
                let prefix = if selected { "›" } else { " " };
                let base = if selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                let kind = Span::styled(
                    format!("{prefix} [{}] ", p.kind),
                    if selected {
                        base
                    } else {
                        Style::default().fg(Color::Yellow)
                    },
                );
                let desc = Span::styled(p.description.clone(), base);
                let url = p
                    .url
                    .as_deref()
                    .map(|u| {
                        Span::styled(
                            format!("  {u}"),
                            if selected {
                                base
                            } else {
                                Style::default().fg(Color::DarkGray)
                            },
                        )
                    })
                    .unwrap_or_else(|| Span::raw(""));
                ListItem::new(Line::from(vec![kind, desc, url]))
            })
            .collect()
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Pending ({}) ", app.pending.len())),
    );
    f.render_widget(list, area);

    if answering {
        let titulo = app
            .answer_input
            .as_ref()
            .map(|(_, _, label)| label.clone())
            .unwrap_or_else(|| "Answer".to_string());
        let input = Paragraph::new(app.editor.render_with_cursor()).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(format!(" {titulo} ")),
        );
        f.render_widget(input, chunks[1]);
    }
}
