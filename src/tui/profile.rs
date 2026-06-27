//! Profile screen: background, base CV and search variants (editable).

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use super::{App, EditField};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    render_fields(f, chunks[0], app);
    render_variants(f, chunks[1], app);
}

fn render_fields(f: &mut Frame, area: Rect, app: &App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_field(
        f,
        cols[0],
        app,
        EditField::Background,
        "Background",
        &app.profile_background,
        "(empty — press [e] to edit)",
    );
    render_field(
        f,
        cols[1],
        app,
        EditField::Cv,
        "Base CV",
        &app.profile_cv,
        "(empty — press [c] to edit)",
    );
}

fn render_field(
    f: &mut Frame,
    area: Rect,
    app: &App,
    field: EditField,
    title: &str,
    value: &str,
    placeholder: &str,
) {
    let editing = app.editing.as_ref() == Some(&field);
    let (text, style, border) = if editing {
        (
            app.editor.render_with_cursor(),
            Style::default().fg(Color::White),
            Color::Yellow,
        )
    } else if value.is_empty() {
        (
            placeholder.to_string(),
            Style::default().fg(Color::DarkGray),
            Color::DarkGray,
        )
    } else {
        (value.to_string(), Style::default(), Color::DarkGray)
    };

    let title = if editing {
        format!(" {title} (editing) ")
    } else {
        format!(" {title} ")
    };

    let p = Paragraph::new(text)
        .style(style)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border))
                .title(title),
        );
    f.render_widget(p, area);
}

fn render_variants(f: &mut Frame, area: Rect, app: &App) {
    // Input line for new variant / import (when applicable).
    let adding = matches!(
        app.editing,
        Some(EditField::NewVariantLabel)
            | Some(EditField::NewVariantQuery)
            | Some(EditField::ImportPath)
            | Some(EditField::ImportLinkedin)
    );
    let constraints = if adding {
        vec![Constraint::Min(0), Constraint::Length(3)]
    } else {
        vec![Constraint::Min(0)]
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let items: Vec<ListItem> = if app.variants.is_empty() {
        vec![ListItem::new(Line::styled(
            "(no variants — press [a] to add)",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        app.variants
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let mark = if v.enabled { "[x]" } else { "[ ]" };
                let selected = i == app.variant_sel;
                let prefix = if selected { "›" } else { " " };
                let style = if selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else if v.enabled {
                    Style::default()
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                ListItem::new(Line::styled(
                    format!("{prefix} {mark} {} — {}", v.label, v.query),
                    style,
                ))
            })
            .collect()
    };
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Search variants ({}) ", app.variants.len())),
    );
    f.render_widget(list, chunks[0]);

    if adding {
        let (label_title, value) = match app.editing {
            Some(EditField::NewVariantLabel) => {
                ("New variant — label", app.editor.render_with_cursor())
            }
            Some(EditField::NewVariantQuery) => (
                "New variant — search query",
                app.editor.render_with_cursor(),
            ),
            Some(EditField::ImportPath) => (
                "Import CV — file path (pdf/docx/txt)",
                app.editor.render_with_cursor(),
            ),
            Some(EditField::ImportLinkedin) => (
                "Import from LinkedIn — URL",
                app.editor.render_with_cursor(),
            ),
            _ => ("", String::new()),
        };
        let input = Paragraph::new(value).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(format!(" {label_title} ")),
        );
        f.render_widget(input, chunks[1]);
    }
}
