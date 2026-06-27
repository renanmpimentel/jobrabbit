//! Session screen: agent stream in real time (replaces the embedded terminal).

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use super::{AgentStatus, App};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let lines: Vec<Line> = app
        .log
        .iter()
        .map(|l| {
            let color = if l.starts_with('✖') {
                Color::Red
            } else if l.starts_with('✔') {
                Color::Green
            } else if l.starts_with('🔧') {
                Color::Yellow
            } else if l.starts_with('▶') || l.starts_with('—') {
                Color::Cyan
            } else {
                Color::Reset
            };
            Line::styled(l.clone(), Style::default().fg(color))
        })
        .collect();

    // Useful internal height (excluding borders).
    let inner_h = area.height.saturating_sub(2);
    let total = lines.len() as u16;

    // In follow mode, anchor at the end.
    let scroll = if app.log_follow {
        total.saturating_sub(inner_h)
    } else {
        app.log_scroll.min(total.saturating_sub(1))
    };

    let title = format!(
        " Session — {} {} ",
        app.agent_status.label(),
        if app.log_follow {
            "[follow]"
        } else {
            "[scroll ↑↓]"
        }
    );
    let border_color = match app.agent_status {
        AgentStatus::Running => Color::Yellow,
        AgentStatus::Error(_) => Color::Red,
        AgentStatus::Idle => Color::DarkGray,
    };

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(title),
        )
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    f.render_widget(p, area);
}
