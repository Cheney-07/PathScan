use crate::app::{App, Focus};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};

pub fn draw_ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(5)])
        .split(f.area());
    draw_header(f, chunks[0], app);
    draw_body(f, chunks[1], app);
    draw_footer(f, chunks[2], app);
}

fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let status_str = match app.status {
        pathscan_core::types::ScanStatus::Running => "Running",
        pathscan_core::types::ScanStatus::Paused => "Paused",
        pathscan_core::types::ScanStatus::Completed => "Completed",
        pathscan_core::types::ScanStatus::Cancelled => "Cancelled",
    };
    let text = format!(
        "PathScan v0.1.0 | Status: {} | {}/{} ({:.0}%)",
        status_str,
        app.completed,
        app.total,
        app.progress_pct() * 100.0
    );
    let gauge = Gauge::default()
        .block(Block::default().title(text).borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Green))
        .ratio(app.progress_pct());
    f.render_widget(gauge, area);
}

fn draw_body(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(area);
    draw_log(f, chunks[0], app);
    draw_stats(f, chunks[1], app);
}

fn draw_log(f: &mut Frame, area: Rect, app: &App) {
    let filtered = app.filtered_results();
    let items: Vec<ListItem> = filtered
        .iter()
        .skip(app.scroll_offset)
        .take(area.height.saturating_sub(2) as usize)
        .map(|r| {
            let c = match r.status {
                200..=299 => Color::Green,
                300..=399 => Color::Yellow,
                400..=499 => Color::Blue,
                _ => Color::Red,
            };
            let redir = r
                .redirect_location
                .as_ref()
                .map(|l| format!(" -> {}", l))
                .unwrap_or_default();
            ListItem::new(Span::styled(
                format!("[{:>3}] {}{}", r.status, r.url, redir),
                Style::default().fg(c),
            ))
        })
        .collect();

    let border = if app.focus == Focus::Log {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    f.render_widget(
        List::new(items).block(
            Block::default()
                .title("Results")
                .borders(Borders::ALL)
                .border_style(border),
        ),
        area,
    );
}

fn draw_stats(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(9)])
        .split(area);

    let (s2xx, s3xx, s4xx, s5xx) = app.status_counts();
    let stats = vec![
        format!("Total:    {}/{}", app.completed, app.total),
        format!("200-299:  {}", s2xx),
        format!("300-399:  {}", s3xx),
        format!("400-499:  {}", s4xx),
        format!("500+:    {}", s5xx),
        format!("Errors:   {}", app.errors.len()),
    ];

    f.render_widget(
        Paragraph::new(stats.join("\n"))
            .block(Block::default().title("Statistics").borders(Borders::ALL)),
        chunks[0],
    );

    let shortcuts = Line::from(vec![
        Span::styled("p", Style::default().fg(Color::Yellow)),
        Span::raw(":pause  "),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(":quit  "),
        Span::styled("s", Style::default().fg(Color::Yellow)),
        Span::raw(":save  "),
        Span::styled("f", Style::default().fg(Color::Yellow)),
        Span::raw(":filter  "),
        Span::styled("Tab", Style::default().fg(Color::Yellow)),
        Span::raw(":focus  "),
        Span::styled("↑↓", Style::default().fg(Color::Yellow)),
        Span::raw(":navigate"),
    ]);

    f.render_widget(
        Paragraph::new(shortcuts).block(Block::default().title("Keys").borders(Borders::ALL)),
        chunks[1],
    );
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let text = app
        .selected_result()
        .map(|r| {
            format!(
                "[{}] {} | Type: {} | Length: {} | Redirect: {}",
                r.status,
                r.url,
                r.content_type.as_deref().unwrap_or("-"),
                r.content_length,
                r.redirect_location.as_deref().unwrap_or("-"),
            )
        })
        .unwrap_or_else(|| "No result selected".to_string());

    f.render_widget(
        Paragraph::new(text).block(Block::default().title("Detail").borders(Borders::ALL)),
        area,
    );
}
