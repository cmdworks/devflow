use colored::*;
use devflow_mcp::{HttpServer, McpHandler, StdioServer};
use std::sync::Arc;

use crate::mcp_connect;

pub async fn handle_mcp_serve(
    http: bool,
    port: u16,
    token: Option<String>,
    agent_hint: Option<String>,
) -> anyhow::Result<()> {
    let handler = Arc::new(McpHandler::new().with_client_hint(agent_hint));

    if http {
        let server = HttpServer::new(handler, port).with_auth(token);
        server.run().await.map_err(|e| anyhow::anyhow!("{}", e))?;
    } else {
        let server = StdioServer::new(handler);
        server.run().await?;
    }

    Ok(())
}

pub async fn handle_mcp_connect(agent: &str, force: bool) -> anyhow::Result<()> {
    println!(
        "\n{}",
        "═══ DevFlow MCP AI Host Auto-Connector ═══".purple().bold()
    );
    let targets = mcp_connect::get_known_host_targets();

    let filter = agent.to_lowercase();
    let mut configured_count = 0;

    for target in &targets {
        let matches = match filter.as_str() {
            "all" => true,
            "claude" => target.name.to_lowercase().contains("claude"),
            "cursor" => target.name.to_lowercase().contains("cursor"),
            "antigravity" | "agy" => target.name.to_lowercase().contains("antigravity"),
            "vscode" | "code" => target.name.to_lowercase().contains("vs code"),
            other => target.name.to_lowercase().contains(other),
        };

        if matches {
            match mcp_connect::configure_agent_target(target, force) {
                Ok(msg) => {
                    println!("  {} {}", "✓".green().bold(), msg);
                    configured_count += 1;
                }
                Err(e) => {
                    println!("  {} Failed for {}: {}", "✗".red().bold(), target.name, e);
                }
            }
        }
    }

    if configured_count == 0 {
        println!("  {}", format!("No matching AI hosts found for '{}'. Available: claude, cursor, antigravity, vscode, all", agent).yellow());
    } else {
        println!("\n{} Connected {} configuration file(s). Restart your AI agent to activate DevFlow MCP tools.", "⚡".cyan().bold(), configured_count);
    }
    println!();

    // Persist to MCP log
    let entry = devflow_mcp::McpAccessLogEntry::new(
        "devflow-cli",
        "cli/connect",
        None,
        None,
        None,
        Some(serde_json::json!({ "agent": agent, "force": force })),
        Some(serde_json::json!({ "status": "ok", "configured_count": configured_count })),
        1,
        "success",
        format!(
            "Connected {} AI host configurations for '{}'",
            configured_count, agent
        ),
        None,
    );
    devflow_mcp::append_entry_to_disk(&entry);

    Ok(())
}

pub async fn handle_mcp_status(json_mode: bool) -> anyhow::Result<()> {
    let tools = devflow_mcp::get_tool_definitions();
    let active_sessions = devflow_core::registry::GlobalRegistry::list_active_sessions();

    // Persist to MCP log
    let entry = devflow_mcp::McpAccessLogEntry::new(
        "devflow-cli",
        "cli/status",
        None,
        None,
        None,
        None,
        Some(serde_json::json!({
            "tools_count": tools.len(),
            "active_sessions_count": active_sessions.len()
        })),
        1,
        "success",
        format!(
            "Checked MCP status: {} tools registered, {} active sessions",
            tools.len(),
            active_sessions.len()
        ),
        None,
    );
    devflow_mcp::append_entry_to_disk(&entry);

    if json_mode {
        let status = serde_json::json!({
            "protocol_version": "2024-11-05",
            "server_name": "devflow",
            "version": env!("CARGO_PKG_VERSION"),
            "tools_count": tools.len(),
            "tools": tools,
            "active_sessions_count": active_sessions.len(),
            "active_sessions": active_sessions,
        });
        println!("{}", serde_json::to_string_pretty(&status)?);
        return Ok(());
    }

    println!(
        "\n{}",
        "═══ DevFlow Model Context Protocol (MCP) Status ═══"
            .purple()
            .bold()
    );
    println!("  Protocol Version : {}", "2024-11-05".cyan());
    println!("  Server Binary    : {}", "devflow mcp serve".green());
    println!(
        "  Available Tools  : {}",
        format!("{} tools registered", tools.len()).bold()
    );
    println!(
        "  Active Sessions  : {} running project target(s)",
        active_sessions.len()
    );
    println!("\n{}", "Registered MCP Tools:".bold());
    for t in &tools {
        let name = t.get("name").and_then(|n| n.as_str()).unwrap_or("");
        let desc = t.get("description").and_then(|d| d.as_str()).unwrap_or("");
        println!("  • {} : {}", name.cyan().bold(), desc.dimmed());
    }
    println!(
        "\nTip: Run '{}' to auto-configure Claude, Cursor, Antigravity, or VS Code.",
        "devflow mcp connect all".bold()
    );
    println!(
        "     Run '{}' to start the stdio server.\n",
        "devflow mcp serve".bold()
    );
    Ok(())
}

pub async fn handle_mcp_tools(json_mode: bool) -> anyhow::Result<()> {
    let tools = devflow_mcp::get_tool_definitions();

    // Persist to MCP log
    let entry = devflow_mcp::McpAccessLogEntry::new(
        "devflow-cli",
        "cli/tools",
        None,
        None,
        None,
        None,
        Some(serde_json::json!({ "tools_count": tools.len() })),
        1,
        "success",
        format!("Listed {} MCP tool definitions", tools.len()),
        None,
    );
    devflow_mcp::append_entry_to_disk(&entry);

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&tools)?);
        return Ok(());
    }

    println!(
        "\n{}",
        "═══ DevFlow MCP Tool Catalog (11 Tools) ═══".cyan().bold()
    );
    for (i, t) in tools.iter().enumerate() {
        let name = t.get("name").and_then(|n| n.as_str()).unwrap_or("");
        let desc = t.get("description").and_then(|d| d.as_str()).unwrap_or("");
        let schema = t.get("inputSchema");
        let props = schema
            .and_then(|s| s.get("properties"))
            .and_then(|p| p.as_object());
        let required = schema
            .and_then(|s| s.get("required"))
            .and_then(|r| r.as_array());

        println!(
            "\n[{}] {}",
            (i + 1).to_string().bold(),
            name.purple().bold()
        );
        println!("    {}", desc);
        if let Some(props_map) = props {
            if !props_map.is_empty() {
                println!("    Parameters:");
                for (pname, pinfo) in props_map {
                    let ptype = pinfo.get("type").and_then(|v| v.as_str()).unwrap_or("any");
                    let pdesc = pinfo
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let is_req =
                        required.is_some_and(|reqs| reqs.iter().any(|r| r.as_str() == Some(pname)));
                    let req_tag = if is_req {
                        "[required]".red()
                    } else {
                        "[optional]".dimmed()
                    };
                    println!(
                        "      - {} ({}) {}: {}",
                        pname.cyan(),
                        ptype,
                        req_tag,
                        pdesc
                    );
                }
            } else {
                println!("    No input parameters required");
            }
        }
    }
    println!();
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_mcp_logs(
    limit: usize,
    session: Option<String>,
    agent: Option<String>,
    tool: Option<String>,
    status: Option<String>,
    full: bool,
    clear: bool,
    delete: Option<String>,
) -> anyhow::Result<()> {
    if clear {
        if let Some(ref sess) = session {
            devflow_mcp::delete_access_logs_by_session(sess);
            println!(
                "{} Cleared all MCP logs for session '{}'.",
                "✓".green().bold(),
                sess.cyan()
            );
        } else {
            devflow_mcp::clear_all_access_logs();
            println!(
                "{} Cleared all MCP logs from SQLite store.",
                "✓".green().bold()
            );
        }
        return Ok(());
    }

    if let Some(ref entry_id) = delete {
        let deleted = devflow_mcp::delete_access_log(entry_id);
        if deleted {
            println!(
                "{} Successfully deleted MCP log entry '{}'.",
                "✓".green().bold(),
                entry_id.cyan()
            );
        } else {
            println!(
                "{} MCP log entry '{}' not found.",
                "!".yellow().bold(),
                entry_id
            );
        }
        return Ok(());
    }

    let filter = devflow_mcp::db::McpLogFilter {
        limit: Some(limit),
        session_id: session.clone(),
        client: agent.clone(),
        tool_name: tool.clone(),
        status: status.clone(),
        ..Default::default()
    };

    let entries = devflow_mcp::query_access_logs(&filter);

    let mut filter_parts = Vec::new();
    if let Some(ref s) = session {
        filter_parts.push(format!("session: {}", s));
    }
    if let Some(ref a) = agent {
        filter_parts.push(format!("agent: {}", a));
    }
    if let Some(ref t) = tool {
        filter_parts.push(format!("tool: {}", t));
    }
    let filter_desc = if filter_parts.is_empty() {
        String::new()
    } else {
        format!(" | {}", filter_parts.join(", "))
    };
    println!(
        "\n{}",
        format!(
            "═══ DevFlow MCP Access & Trace Logs ({} entries{}) ═══",
            entries.len(),
            filter_desc
        )
        .purple()
        .bold()
    );
    if entries.is_empty() {
        println!(
            "  {}",
            "No matching MCP tool calls or CLI activities found.".dimmed()
        );
        println!("  Tip: Trigger tools via an AI host (Cursor, Claude, Antigravity) or run 'devflow mcp status'.\n");
        return Ok(());
    }

    for (i, entry) in entries.iter().enumerate() {
        let status_colored = if entry.status == "success" {
            "✓ SUCCESS".green().bold()
        } else {
            "✗ FAILED".red().bold()
        };
        let target_label = entry.tool_name.as_deref().unwrap_or(&entry.method);
        let session_label = entry.session_id.as_deref().unwrap_or("standalone");
        let short_id = if entry.id.len() > 8 {
            &entry.id[..8]
        } else {
            &entry.id
        };

        println!(
            "[{}] {} | {} | {} ({}ms) [{}]",
            (i + 1).to_string().dimmed(),
            entry.timestamp.dimmed(),
            status_colored,
            target_label.cyan().bold(),
            entry.duration_ms,
            short_id.dimmed()
        );
        let client_colored = match entry.client.to_lowercase().as_str() {
            c if c.contains("cursor") => entry.client.cyan().bold(),
            c if c.contains("claude") => entry.client.yellow().bold(),
            c if c.contains("antigravity") => entry.client.purple().bold(),
            c if c.contains("code") => entry.client.blue().bold(),
            _ => entry.client.green(),
        };
        println!(
            "     Agent: {} | Session: {} | Path: {}",
            client_colored,
            session_label.bold(),
            entry.project_path.as_deref().unwrap_or("-").dimmed()
        );
        println!("     Summary: {}", entry.summary);
        if let Some(err) = &entry.error_message {
            println!("     Error: {}", err.red());
        }

        if full {
            if let Some(ref args) = entry.arguments {
                if let Ok(formatted) = serde_json::to_string_pretty(args) {
                    println!(
                        "     Input Arguments:\n{}",
                        formatted
                            .lines()
                            .map(|l| format!("       {}", l))
                            .collect::<Vec<_>>()
                            .join("\n")
                            .dimmed()
                    );
                }
            }
            if let Some(ref resp) = entry.response {
                if let Ok(formatted) = serde_json::to_string_pretty(resp) {
                    println!(
                        "     Output Response:\n{}",
                        formatted
                            .lines()
                            .map(|l| format!("       {}", l))
                            .collect::<Vec<_>>()
                            .join("\n")
                            .dimmed()
                    );
                }
            }
        }
    }

    println!();
    if !full {
        println!(
            "  Tip: Add {} to view full input arguments & response payloads.",
            "--full".cyan().bold()
        );
    }
    println!(
        "  Tip: Filter by session: {}, clear logs: {}, delete entry: {}\n",
        "devflow mcp logs --session <id>".bold(),
        "devflow mcp logs --clear".bold(),
        "devflow mcp logs --delete <id>".bold()
    );
    Ok(())
}
