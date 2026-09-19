use crate::app::{AppMode, TuiApp};
use crate::ui;
use crossterm::{
    event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use devflow_frameworks::session::SessionManager;
use futures::StreamExt;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;

pub struct TuiRunner;

impl TuiRunner {
    /// Run TUI Hub for workspace (Zero-arg devflow entry)
    pub async fn run_hub(dir: PathBuf) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut app = TuiApp::new_hub(dir).await;
        let mut reader = EventStream::new();
        let mut tick_interval = interval(Duration::from_millis(50));
        let mut sync_interval = interval(Duration::from_millis(1000));

        let mut event_bus_rx: Option<tokio::sync::broadcast::Receiver<devflow_core::event::DevflowEvent>> = None;

        loop {
            terminal.draw(|f| ui::render(f, &app))?;

            if app.should_quit {
                break;
            }

            tokio::select! {
                _ = tick_interval.tick() => {}

                _ = sync_interval.tick() => {
                    app.refresh_hub();
                }

                Some(Ok(event)) = reader.next() => {
                    match event {
                        Event::Key(key) if key.kind == KeyEventKind::Press => {
                            match key.code {
                                KeyCode::Char('q') => {
                                    if app.mode == AppMode::Hub {
                                        app.should_quit = true;
                                    } else {
                                        app.mode = AppMode::Hub;
                                    }
                                }
                                KeyCode::Esc => {
                                    if app.mode == AppMode::Session {
                                        app.mode = AppMode::Hub;
                                    } else {
                                        app.should_quit = true;
                                    }
                                }
                                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                    app.should_quit = true;
                                }
                                KeyCode::Enter => {
                                    if app.mode == AppMode::Hub {
                                        if let Ok(_) = app.start_selected_target().await {
                                            if let Some(ref sess) = app.session {
                                                event_bus_rx = Some(sess.event_bus.subscribe());
                                            }
                                        }
                                    }
                                }
                                KeyCode::Char('r') => {
                                    app.reload().await;
                                }
                                KeyCode::Char('R') => {
                                    app.restart().await;
                                }
                                KeyCode::Char('d') => {
                                    app.run_doctor().await;
                                }
                                KeyCode::Char('l') => {
                                    if app.mode == AppMode::Session {
                                        app.toggle_log_level();
                                    }
                                }
                                KeyCode::Tab => {
                                    app.next_panel();
                                }
                                KeyCode::Up | KeyCode::Char('k') => {
                                    app.nav_up();
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    app.nav_down();
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }

                recv_res = async {
                    if let Some(ref mut rx) = event_bus_rx {
                        rx.recv().await
                    } else {
                        futures::future::pending().await
                    }
                } => {
                    match recv_res {
                        Ok(evt) => app.handle_event(evt),
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {}
                    }
                }
            }
        }

        if let Some(ref sess) = app.session {
            let _ = sess.stop().await;
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        Ok(())
    }

    /// Run TUI directly attached to an active session (`devflow dev`)
    pub async fn run(session: Arc<SessionManager>) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut app = TuiApp::new(session.clone()).await;
        let mut event_bus_rx = session.event_bus.subscribe();
        let mut reader = EventStream::new();
        let mut tick_interval = interval(Duration::from_millis(50));

        loop {
            terminal.draw(|f| ui::render(f, &app))?;

            if app.should_quit {
                break;
            }

            tokio::select! {
                _ = tick_interval.tick() => {}

                recv_res = event_bus_rx.recv() => {
                    match recv_res {
                        Ok(event) => app.handle_event(event),
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {}
                    }
                }

                Some(Ok(event)) = reader.next() => {
                    match event {
                        Event::Key(key) if key.kind == KeyEventKind::Press => {
                            match key.code {
                                KeyCode::Char('q') | KeyCode::Esc => {
                                    app.should_quit = true;
                                }
                                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                    app.should_quit = true;
                                }
                                KeyCode::Char('r') => {
                                    app.reload().await;
                                }
                                KeyCode::Char('R') => {
                                    app.restart().await;
                                }
                                KeyCode::Char('d') => {
                                    app.run_doctor().await;
                                }
                                KeyCode::Char('l') => {
                                    app.toggle_log_level();
                                }
                                KeyCode::Tab => {
                                    app.next_panel();
                                }
                                KeyCode::Up | KeyCode::Char('k') => {
                                    app.scroll_up();
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    app.scroll_down();
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        Ok(())
    }
}
