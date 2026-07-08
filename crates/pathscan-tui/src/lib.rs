mod app;
mod widgets;

use app::{App, Focus};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use pathscan_core::engine::ScanEngine;
use pathscan_core::types::{ScanConfig, ScanStatus};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use tokio::sync::mpsc;

pub fn run() -> io::Result<()> {
    let config = ScanConfig::new("http://localhost".into());

    let (tx, rx) = mpsc::unbounded_channel();
    let _scan = tokio::spawn(async move { ScanEngine::run(config, Some(tx)).await });

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(rx, 0);
    let mut should_quit = false;

    while !should_quit {
        app.update();
        terminal.draw(|f| widgets::draw_ui(f, &app))?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => should_quit = true,
                        KeyCode::Char('p') => {
                            app.status = match app.status {
                                ScanStatus::Running => ScanStatus::Paused,
                                ScanStatus::Paused => ScanStatus::Running,
                                s => s,
                            };
                        }
                        KeyCode::Tab => {
                            app.focus = match app.focus {
                                Focus::Log => Focus::Stats,
                                Focus::Stats => Focus::Log,
                            };
                        }
                        KeyCode::Up if app.selected_index > 0 => {
                            app.selected_index -= 1;
                        }
                        KeyCode::Down if app.selected_index + 1 < app.results.len() => {
                            app.selected_index += 1;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
