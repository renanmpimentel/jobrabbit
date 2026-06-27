//! Config screen: edits all settings + the answer bank.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use super::{App, EditField, SETTING_KINDS};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let editing = app.editing.as_ref() == Some(&EditField::ConfigInput);
    let chunks = if editing {
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

    let n_settings = SETTING_KINDS.len();
    let mut items: Vec<ListItem> = Vec::new();

    items.push(section("Settings"));
    for (i, kind) in SETTING_KINDS.iter().enumerate() {
        items.push(kv_item(
            kind.label(),
            &kind.value_str(&app.settings),
            i == app.config_sel,
        ));
    }

    items.push(ListItem::new(Line::from("")));
    items.push(section("Answer bank (screening)"));
    for (j, ans) in app.answers.iter().enumerate() {
        let selected = app.config_sel == n_settings + j;
        let value = if ans.value.is_empty() {
            "(empty)".to_string()
        } else {
            ans.value.clone()
        };
        items.push(kv_item(&ans.label, &value, selected));
    }

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Config — [↑↓] navigate  [Enter] edit  [space] toggle "),
    );
    f.render_widget(list, chunks[0]);

    if editing {
        let title = match &app.config_edit {
            Some(super::ConfigTarget::Setting(k)) => k.label().to_string(),
            Some(super::ConfigTarget::Answer { label, .. }) => label.clone(),
            None => "Edit".to_string(),
        };
        let input = Paragraph::new(app.editor.render_with_cursor()).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(format!(" {title} — [Enter] save  [Esc] cancel ")),
        );
        f.render_widget(input, chunks[1]);
    }
}

fn section(title: &str) -> ListItem<'static> {
    ListItem::new(Line::styled(
        format!("── {title} ──"),
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    ))
}

fn kv_item(label: &str, value: &str, selected: bool) -> ListItem<'static> {
    let prefix = if selected { "› " } else { "  " };
    let style = if selected {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    ListItem::new(Line::from(vec![
        Span::styled(format!("{prefix}{label}: "), style),
        Span::styled(value.to_string(), style.fg(Color::White)),
    ]))
}
