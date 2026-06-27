//! Dashboard screen: metrics + pipeline chart.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Paragraph},
    Frame,
};

use super::App;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(0)])
        .split(area);

    render_cards(f, chunks[0], app);

    let charts = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(chunks[1]);
    render_pipeline(f, charts[0], app);
    render_apps_per_day(f, charts[1], app);
}

fn render_apps_per_day(f: &mut Frame, area: Rect, app: &App) {
    let bars: Vec<Bar> = app
        .apps_per_day
        .iter()
        .map(|(label, n)| {
            Bar::default()
                .label(Line::from(label.clone()))
                .value(*n)
                .style(Style::default().fg(Color::Green))
                .value_style(Style::default().fg(Color::Black).bg(Color::Green))
        })
        .collect();
    let chart = BarChart::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Applications / day (7d) "),
        )
        .data(BarGroup::default().bars(&bars))
        .bar_width(5)
        .bar_gap(1);
    f.render_widget(chart, area);
}

fn render_cards(f: &mut Frame, area: Rect, app: &App) {
    let s = &app.stats;
    let lines = vec![
        Line::from(vec![
            Span::styled("Jobs found: ", Style::default().fg(Color::Gray)),
            Span::styled(
                s.total_jobs.to_string(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Applications:      ", Style::default().fg(Color::Gray)),
            Span::styled(
                s.total_applications.to_string(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Applied:           ", Style::default().fg(Color::Gray)),
            Span::styled(
                s.applied.to_string(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Pending:           ", Style::default().fg(Color::Gray)),
            Span::styled(
                s.pending_actions.to_string(),
                Style::default()
                    .fg(if s.pending_actions > 0 {
                        Color::Red
                    } else {
                        Color::Green
                    })
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];
    let p = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Summary "));
    f.render_widget(p, area);
}

fn render_pipeline(f: &mut Frame, area: Rect, app: &App) {
    let s = &app.stats;
    let bars = vec![
        bar("Jobs", s.total_jobs as u64, Color::Cyan),
        bar("Apps.", s.total_applications as u64, Color::Blue),
        bar("Applied", s.applied as u64, Color::Green),
        bar("Pending", s.pending_actions as u64, Color::Red),
    ];
    let chart = BarChart::default()
        .block(Block::default().borders(Borders::ALL).title(" Pipeline "))
        .data(BarGroup::default().bars(&bars))
        .bar_width(11)
        .bar_gap(3);
    f.render_widget(chart, area);
}

fn bar(label: &str, value: u64, color: Color) -> Bar<'_> {
    Bar::default()
        .label(Line::from(label.to_string()))
        .value(value)
        .style(Style::default().fg(color))
        .value_style(Style::default().fg(Color::Black).bg(color))
}
