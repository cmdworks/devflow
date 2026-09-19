use crate::app::{AppMode, TuiApp};
use crate::ui;
use crossterm::{
    event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use devflow_core::event::DevflowEvent;
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

        let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<(usize, DevflowEvent)>(2000);

        let spawn_event_forwarder = |target_idx: usize, mut rx: tokio::sync::broadcast::Receiver<DevflowEvent>, tx: tokio::sync::mpsc::Sender<(usize, DevflowEvent)>| {
            tokio::spawn(async move {
                loop {
                    match rx.recv().await {
                        Ok(evt) => {
                            if tx.send((target_idx, evt)).await.is_err() {
                                break;
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
            });
        };

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

                Some((target_idx, evt)) = event_rx.recv() => {
                    app.handle_target_event(target_idx, evt);
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
                                // Direct tab jumping 1-9
                                KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
                                    let digit = (c as u8 - b'1') as usize;
                                    if app.mode == AppMode::Session {
                                        app.select_tab(digit);
                                    }
                                }
                                KeyCode::Char('h') | KeyCode::Left => {
                                    if app.mode == AppMode::Session {
                                        app.prev_tab();
                                    }
                                }
                                KeyCode::Char('l') | KeyCode::Right => {
                                    if app.mode == AppMode::Session {
                                        app.next_tab();
                                    }
                                }
                                KeyCode::Tab => {
                                    if app.mode == AppMode::Hub {
                                        app.next_panel();
                                    } else {
                                        app.next_tab();
                                    }
                                }
                                KeyCode::Enter | KeyCode::Char('s') => {
                                    if app.mode == AppMode::Hub {
                                        if let Ok((target_idx, rx)) = app.start_selected_target().await {
                                            spawn_event_forwarder(target_idx, rx, event_tx.clone());
                                        }
                                    } else if app.active_tab_idx < app.target_states.len() {
                                        let idx = app.active_tab_idx;
                                        if app.target_states[idx].session.is_some() {
                                            app.stop_target(idx).await;
                                        } else if let Ok(rx) = app.start_target(idx).await {
                                            spawn_event_forwarder(idx, rx, event_tx.clone());
                                        }
                                    }
                                }
                                KeyCode::Char('a') => {
                                    let launched = app.start_all_targets().await;
                                    for (target_idx, rx) in launched {
                                        spawn_event_forwarder(target_idx, rx, event_tx.clone());
                                    }
                                }
                                KeyCode::Char('x') => {
                                    if app.mode == AppMode::Session {
                                        if app.active_tab_idx < app.target_states.len() {
                                            app.stop_target(app.active_tab_idx).await;
                                        } else {
                                            app.stop_all_targets().await;
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
                                KeyCode::Char('f') => {
                                    if app.mode == AppMode::Session {
                                        app.toggle_autoscroll();
                                    }
                                }
                                KeyCode::PageUp | KeyCode::Char('K') => {
                                    if app.mode == AppMode::Session {
                                        app.scroll_up(10);
                                    }
                                }
                                KeyCode::PageDown | KeyCode::Char('J') => {
                                    if app.mode == AppMode::Session {
                                        app.scroll_down(10);
                                    }
                                }
                                KeyCode::Home | KeyCode::Char('g') => {
                                    if app.mode == AppMode::Session {
                                        app.scroll_to_top();
                                    }
                                }
                                KeyCode::End | KeyCode::Char('G') => {
                                    if app.mode == AppMode::Session {
                                        app.scroll_to_bottom();
                                    }
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
            }
        }

        app.stop_all_targets().await;

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
                        Ok(event) => app.handle_target_event(0, event),
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
                                KeyCode::Char('f') => {
                                    app.toggle_autoscroll();
                                }
                                KeyCode::PageUp | KeyCode::Char('K') => {
                                    app.scroll_up(10);
                                }
                                KeyCode::PageDown | KeyCode::Char('J') => {
                                    app.scroll_down(10);
                                }
                                KeyCode::Home | KeyCode::Char('g') => {
                                    app.scroll_to_top();
                                }
                                KeyCode::End | KeyCode::Char('G') => {
                                    app.scroll_to_bottom();
                                }
                                KeyCode::Tab => {
                                    app.next_tab();
                                }
                                KeyCode::Up | KeyCode::Char('k') => {
                                    app.scroll_up(1);
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    app.scroll_down(1);
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
