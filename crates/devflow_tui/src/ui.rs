use crate::app::{AppMode, HubFocus, SessionPanel, TuiApp};
use devflow_protocol::{LogLevel, SessionStatus};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn render(f: &mut Frame, app: &TuiApp) {
    match app.mode {
        AppMode::Hub => render_hub(f, app),
        AppMode::Session => render_session(f, app),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Hub Mode: Project Target Hub & Cross-Terminal Multi-Session Dashboard
// ═══════════════════════════════════════════════════════════════════════════

fn render_hub(f: &mut Frame, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header / Status Bar
            Constraint::Min(12),   // Main Content Area
            Constraint::Length(3), // Footer / Keymap
        ])
        .split(f.area());

    render_hub_header(f, app, chunks[0]);
    render_hub_main(f, app, chunks[1]);
    render_hub_footer(f, app, chunks[2]);
}

fn render_hub_header(f: &mut Frame, app: &TuiApp, area: Rect) {
    let dir_name = app.project_dir.file_name().and_then(|s| s.to_str()).unwrap_or("Workspace");
    let active_count = app.active_sessions.len();

    let session_badge = if active_count > 0 {
        Span::styled(format!(" {} Active Session{} ", active_count, if active_count > 1 { "s" } else { "" }), Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD))
    } else {
        Span::styled(" 0 Active Sessions ", Style::default().fg(Color::DarkGray))
    };

    let status_msg = app.status_message.as_deref().unwrap_or("Ready");

    let header_text = vec![Line::from(vec![
        Span::styled(" ⚡ DevFlow Hub ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(format!("Folder: {}", dir_name), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw("  |  "),
        session_badge,
        Span::raw("  |  "),
        Span::styled(format!("Status: {}", status_msg), Style::default().fg(Color::Yellow)),
    ])];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(header_text).block(block);
    f.render_widget(paragraph, area);
}

fn render_hub_main(f: &mut Frame, app: &TuiApp, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Left: Targets + Active Sessions
            Constraint::Percentage(50), // Right: Target Details/Actions + Devices
        ])
        .split(area);

    let left_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(55), // Targets panel
            Constraint::Percentage(45), // Cross-terminal sessions panel
        ])
        .split(cols[0]);

    let right_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(55), // Action & Target Details
            Constraint::Percentage(45), // Devices panel
        ])
        .split(cols[1]);

    render_hub_targets(f, app, left_rows[0]);
    render_hub_sessions(f, app, left_rows[1]);
    render_hub_actions(f, app, right_rows[0]);
    render_hub_devices(f, app, right_rows[1]);
}

fn render_hub_targets(f: &mut Frame, app: &TuiApp, area: Rect) {
    let is_focused = app.hub_focus == HubFocus::Targets;
    let border_color = if is_focused { Color::Cyan } else { Color::DarkGray };

    let items: Vec<ListItem> = if app.targets.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(" No local targets found. Showing known projects below.", Style::default().fg(Color::DarkGray))))]
    } else {
        app.targets
            .iter()
            .enumerate()
            .map(|(i, target)| {
                let is_selected = i == app.selected_target_idx;
                let marker = if is_selected { "▶ " } else { "  " };
                let marker_color = if is_selected { Color::Cyan } else { Color::DarkGray };
                let name_style = if is_selected {
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                };

                let line = Line::from(vec![
                    Span::styled(marker, Style::default().fg(marker_color).add_modifier(Modifier::BOLD)),
                    Span::styled(&target.name, name_style),
                    Span::raw(" "),
                    Span::styled(format!("[{}]", target.platform), Style::default().fg(Color::Magenta)),
                    Span::raw(" "),
                    Span::styled(format!("({})", target.framework), Style::default().fg(Color::DarkGray)),
                ]);
                ListItem::new(line)
            })
            .collect()
    };

    let block = Block::default()
        .title(" 1. Runnable Project Targets ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn render_hub_sessions(f: &mut Frame, app: &TuiApp, area: Rect) {
    let is_focused = app.hub_focus == HubFocus::Sessions;
    let border_color = if is_focused { Color::Cyan } else { Color::DarkGray };

    let items: Vec<ListItem> = if app.active_sessions.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(" No other sessions running on system.", Style::default().fg(Color::DarkGray))))]
    } else {
        app.active_sessions
            .iter()
            .enumerate()
            .map(|(i, sess)| {
                let is_selected = i == app.selected_session_idx;
                let marker = if is_selected { "● " } else { "○ " };
                let status_color = match sess.status {
                    SessionStatus::Running => Color::Green,
                    SessionStatus::Building | SessionStatus::Installing | SessionStatus::Launching => Color::Cyan,
                    SessionStatus::Reloading | SessionStatus::Restarting => Color::Yellow,
                    SessionStatus::Failed => Color::Red,
                    _ => Color::DarkGray,
                };

                let line = Line::from(vec![
                    Span::styled(marker, Style::default().fg(status_color)),
                    Span::styled(&sess.project_name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::styled(format!(" [{}]", sess.status), Style::default().fg(status_color)),
                    Span::styled(format!(" PID:{}", sess.pid), Style::default().fg(Color::DarkGray)),
                    Span::styled(format!(" (Reloads: {})", sess.reload_count), Style::default().fg(Color::LightYellow)),
                ]);
                ListItem::new(line)
            })
            .collect()
    };

    let block = Block::default()
        .title(" 2. Active Sessions Across Terminals (Live Sync) ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn render_hub_actions(f: &mut Frame, app: &TuiApp, area: Rect) {
    let mut lines = Vec::new();

    if let Some(target) = app.targets.get(app.selected_target_idx) {
        lines.push(Line::from(vec![
            Span::styled("Selected Target: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&target.name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Platform: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", target.platform), Style::default().fg(Color::Magenta)),
            Span::styled(" | Framework: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&target.framework, Style::default().fg(Color::LightCyan)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Path: ", Style::default().fg(Color::DarkGray)),
            Span::styled(target.path.display().to_string(), Style::default().fg(Color::White)),
        ]));
    } else if let Some(proj) = app.known_projects.get(app.selected_project_idx) {
        lines.push(Line::from(vec![
            Span::styled("Known Project: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&proj.name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Path: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&proj.path, Style::default().fg(Color::White)),
        ]));
    } else {
        lines.push(Line::from(Span::styled("No target or project selected.", Style::default().fg(Color::DarkGray))));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("Quick Actions:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))));
    lines.push(Line::from(vec![
        Span::styled(" [Enter] ", Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(" Launch Live Dev Session (Build → Install → Launch → Stream)"),
    ]));
    lines.push(Line::from(vec![
        Span::styled(" [r]     ", Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" Send Reload Signal to Active Session (IPC)"),
    ]));
    lines.push(Line::from(vec![
        Span::styled(" [R]     ", Style::default().fg(Color::Black).bg(Color::LightYellow).add_modifier(Modifier::BOLD)),
        Span::raw(" Send Full App Restart Signal (IPC)"),
    ]));
    lines.push(Line::from(vec![
        Span::styled(" [d]     ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" Run Doctor Toolchain Diagnostics"),
    ]));

    let block = Block::default()
        .title(" Target Overview & Actions ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn render_hub_devices(f: &mut Frame, app: &TuiApp, area: Rect) {
    let is_focused = app.hub_focus == HubFocus::Devices;
    let border_color = if is_focused { Color::Cyan } else { Color::DarkGray };

    let items: Vec<ListItem> = app
        .devices
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let is_selected = i == app.selected_device_idx;
            let marker = if is_selected { "● " } else { "○ " };
            let marker_color = if is_selected { Color::Green } else { Color::DarkGray };

            let text = Line::from(vec![
                Span::styled(marker, Style::default().fg(marker_color)),
                Span::styled(&d.name, Style::default().fg(if is_selected { Color::White } else { Color::Gray }).add_modifier(if is_selected { Modifier::BOLD } else { Modifier::empty() })),
                Span::styled(format!(" [{}]", d.platform), Style::default().fg(Color::DarkGray)),
            ]);
            ListItem::new(text)
        })
        .collect();

    let block = Block::default()
        .title(" Target Devices & Hardware ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn render_hub_footer(f: &mut Frame, _app: &TuiApp, area: Rect) {
    let footer_spans = vec![
        Span::styled(" [Enter] ", Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(" Run Dev  "),
        Span::styled(" [Tab] ", Style::default().fg(Color::Black).bg(Color::LightBlue).add_modifier(Modifier::BOLD)),
        Span::raw(" Switch Section  "),
        Span::styled(" [↑/↓] ", Style::default().fg(Color::Black).bg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::raw(" Navigate  "),
        Span::styled(" [r] ", Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" Reload  "),
        Span::styled(" [R] ", Style::default().fg(Color::Black).bg(Color::LightYellow).add_modifier(Modifier::BOLD)),
        Span::raw(" Restart  "),
        Span::styled(" [d] ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" Doctor  "),
        Span::styled(" [q] ", Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::raw(" Quit"),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(Line::from(footer_spans)).block(block);
    f.render_widget(paragraph, area);
}

// ═══════════════════════════════════════════════════════════════════════════
// Session Mode: Live Development Runner (Logs, Watcher, Devices)
// ═══════════════════════════════════════════════════════════════════════════

fn render_session(f: &mut Frame, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header / Status Bar
            Constraint::Min(10),   // Main Content Area
            Constraint::Length(3), // Footer / Keymap
        ])
        .split(f.area());

    render_session_header(f, app, chunks[0]);
    render_session_main(f, app, chunks[1]);
    render_session_footer(f, app, chunks[2]);
}

fn render_session_header(f: &mut Frame, app: &TuiApp, area: Rect) {
    let Some(ref sess) = app.session else { return; };
    let state = sess.get_state();

    let status_color = match state.status {
        SessionStatus::Running => Color::Green,
        SessionStatus::Building | SessionStatus::Installing | SessionStatus::Launching => Color::Cyan,
        SessionStatus::Reloading | SessionStatus::Restarting => Color::Yellow,
        SessionStatus::Failed => Color::Red,
        SessionStatus::Stopped | SessionStatus::Idle | SessionStatus::Detecting => Color::DarkGray,
    };

    let target_dev = state.target_device.as_ref().map(|d| d.name.as_str()).unwrap_or("None");

    let header_text = vec![Line::from(vec![
        Span::styled(" ⚡ DevFlow Live ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(format!("Project: {}", state.project_name), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw("  |  "),
        Span::styled(format!("Framework: {}", state.framework), Style::default().fg(Color::Magenta)),
        Span::raw("  |  "),
        Span::styled(format!("Device: {}", target_dev), Style::default().fg(Color::Blue)),
        Span::raw("  |  "),
        Span::styled(format!("Status: {}", state.status), Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
        Span::raw("  |  "),
        Span::styled(format!("Reloads: {}", state.reload_count), Style::default().fg(Color::Yellow)),
        Span::raw("  "),
        Span::styled(format!("Restarts: {}", state.restart_count), Style::default().fg(Color::LightYellow)),
    ])];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(header_text).block(block);
    f.render_widget(paragraph, area);
}

fn render_session_main(f: &mut Frame, app: &TuiApp, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // Left sidebar: Devices + Build/Changes
            Constraint::Percentage(75), // Right main: Logs
        ])
        .split(area);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(45), // Devices panel
            Constraint::Percentage(55), // Changes & Info panel
        ])
        .split(main_chunks[0]);

    render_session_devices_panel(f, app, left_chunks[0]);
    render_session_info_panel(f, app, left_chunks[1]);
    render_session_logs_panel(f, app, main_chunks[1]);
}

fn render_session_devices_panel(f: &mut Frame, app: &TuiApp, area: Rect) {
    let is_focused = app.session_panel == SessionPanel::Devices;
    let border_color = if is_focused { Color::Cyan } else { Color::DarkGray };

    let active_id = app.session.as_ref().and_then(|s| s.get_state().target_device.map(|d| d.id));

    let items: Vec<ListItem> = app
        .devices
        .iter()
        .map(|d| {
            let is_selected = active_id.as_deref() == Some(&d.id);
            let marker = if is_selected { "● " } else { "○ " };
            let marker_color = if is_selected { Color::Green } else { Color::DarkGray };

            let text = Line::from(vec![
                Span::styled(marker, Style::default().fg(marker_color)),
                Span::styled(&d.name, Style::default().fg(if is_selected { Color::White } else { Color::Gray })),
                Span::styled(format!(" ({})", d.platform), Style::default().fg(Color::DarkGray)),
            ]);
            ListItem::new(text)
        })
        .collect();

    let block = Block::default()
        .title(" Devices ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn render_session_info_panel(f: &mut Frame, app: &TuiApp, area: Rect) {
    let is_focused = app.session_panel == SessionPanel::Changes || app.session_panel == SessionPanel::Build;
    let border_color = if is_focused { Color::Cyan } else { Color::DarkGray };

    let mut lines = Vec::new();

    if let Some(ref sess) = app.session {
        let state = sess.get_state();
        if let Some(dur) = state.build_duration_ms {
            lines.push(Line::from(vec![
                Span::styled("Last Build: ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{}ms", dur), Style::default().fg(Color::Green)),
            ]));
        }

        if let Some(ref err) = state.last_error {
            lines.push(Line::from(vec![
                Span::styled("Error: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled(err.clone(), Style::default().fg(Color::LightRed)),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("Recent Changes:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));

    if app.changes.is_empty() {
        lines.push(Line::from(Span::styled(" Watching for changes...", Style::default().fg(Color::DarkGray))));
    } else {
        for change in app.changes.iter().rev().take(6) {
            lines.push(Line::from(Span::styled(format!(" {}", change), Style::default().fg(Color::Yellow))));
        }
    }

    let block = Block::default()
        .title(" Status & Changes ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn render_session_logs_panel(f: &mut Frame, app: &TuiApp, area: Rect) {
    let is_focused = app.session_panel == SessionPanel::Logs;
    let border_color = if is_focused { Color::Cyan } else { Color::DarkGray };

    let lvl_filter_text = match app.log_level_filter {
        Some(lvl) => format!(" [Level: {}] ", lvl),
        None => " [All Levels] ".to_string(),
    };

    let title = format!(" Logs{} ", lvl_filter_text);

    let max_lines = area.height.saturating_sub(2) as usize;
    let total_logs = app.logs.len();

    let (start, end) = if total_logs > max_lines {
        let skip_from_end = app.log_scroll_offset;
        let start = total_logs.saturating_sub(max_lines + skip_from_end);
        let end = total_logs.saturating_sub(skip_from_end);
        (start, end)
    } else {
        (0, total_logs)
    };

    let items: Vec<ListItem> = app
        .logs
        .range(start..end)
        .map(|entry| {
            let (level_badge, badge_color) = match entry.level {
                LogLevel::E => (" ERR ", Color::Red),
                LogLevel::W => (" WRN ", Color::Yellow),
                LogLevel::I => (" INF ", Color::Green),
                LogLevel::D => (" DBG ", Color::Blue),
            };

            let time_str = entry.timestamp.format("%H:%M:%S%.3f").to_string();
            let tag_str = entry.tag.as_deref().unwrap_or("app");

            let line = Line::from(vec![
                Span::styled(format!("{} ", time_str), Style::default().fg(Color::DarkGray)),
                Span::styled(level_badge, Style::default().fg(Color::Black).bg(badge_color).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" [{}] ", tag_str), Style::default().fg(Color::LightCyan)),
                Span::styled(&entry.message, Style::default().fg(Color::White)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn render_session_footer(f: &mut Frame, _app: &TuiApp, area: Rect) {
    let footer_spans = vec![
        Span::styled(" [Esc] ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" Back to Hub  "),
        Span::styled(" [r] ", Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" Reload  "),
        Span::styled(" [R] ", Style::default().fg(Color::Black).bg(Color::LightYellow).add_modifier(Modifier::BOLD)),
        Span::raw(" Restart  "),
        Span::styled(" [l] ", Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(" Filter Level  "),
        Span::styled(" [Tab] ", Style::default().fg(Color::Black).bg(Color::LightBlue).add_modifier(Modifier::BOLD)),
        Span::raw(" Switch Panel  "),
        Span::styled(" [↑/↓] ", Style::default().fg(Color::Black).bg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::raw(" Scroll  "),
        Span::styled(" [q] ", Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::raw(" Quit"),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(Line::from(footer_spans)).block(block);
    f.render_widget(paragraph, area);
}
