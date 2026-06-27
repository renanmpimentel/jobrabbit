//! ATS screen: resume evaluation (resume checker) — score + report.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
    Frame,
};

use super::{App, EditField};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let editing = app.editing.as_ref() == Some(&EditField::AtsTarget);

    // gauge (3) | content (min) | [input (3) if pasting]
    let mut constraints = vec![Constraint::Length(3), Constraint::Min(0)];
    if editing {
        constraints.push(Constraint::Length(3));
    }
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    // Score gauge.
    let (score, target) = app
        .cv_review
        .as_ref()
        .map(|r| (r.score.clamp(0, 100) as u16, r.target.clone()))
        .unwrap_or((0, "—".to_string()));
    let color = if score < 50 {
        Color::Red
    } else if score < 75 {
        Color::Yellow
    } else {
        Color::Green
    };
    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" ATS Score — target: {target} ")),
        )
        .gauge_style(Style::default().fg(color))
        .percent(score)
        .label(format!("{score}/100"));
    f.render_widget(gauge, chunks[0]);

    // Content: report + list of target jobs side by side.
    let mid = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(chunks[1]);

    let report = app
        .cv_review
        .as_ref()
        .map(|r| r.report.clone())
        .unwrap_or_else(|| {
            "No evaluation yet.\n\n[g] evaluates resume in general.\n[↑↓] chooses a job and [Enter] evaluates the match.\n[t] pastes a job description.".to_string()
        });
    let p = Paragraph::new(report)
        .wrap(Wrap { trim: false })
        .block(Block::default().borders(Borders::ALL).title(" Report "));
    f.render_widget(p, mid[0]);

    let items: Vec<ListItem> = if app.jobs.is_empty() {
        vec![ListItem::new(Line::styled(
            "(no jobs — run the search)",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        app.jobs
            .iter()
            .enumerate()
            .map(|(i, j)| {
                let sel = i == app.ats_job_sel;
                let prefix = if sel { "› " } else { "  " };
                let style = if sel {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(Line::styled(
                    format!("{prefix}{} @ {}", j.title, j.company),
                    style,
                ))
            })
            .collect()
    };
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Target job (↑↓ Enter) "),
    );
    f.render_widget(list, mid[1]);

    if editing {
        let input = Paragraph::new(app.editor.render_with_cursor()).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" Paste job description — [Enter] evaluate  [Esc] cancel "),
        );
        f.render_widget(input, chunks[2]);
    }
}
