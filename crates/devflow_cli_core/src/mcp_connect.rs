use colored::*;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct HostConfigTarget {
    pub name: &'static str,
    pub path: PathBuf,
    pub server_key: &'static str, // "mcpServers" or "servers"
    pub format: HostFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostFormat {
    CommandArgs, // { "command": "devflow", "args": ["mcp", "serve"] }
}

pub fn get_known_host_targets() -> Vec<HostConfigTarget> {
    let mut targets = Vec::new();
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

    // 1. Claude Desktop
    #[cfg(target_os = "macos")]
    let claude_path = home.join("Library/Application Support/Claude/claude_desktop_config.json");
    #[cfg(target_os = "windows")]
    let claude_path = dirs::config_dir()
        .unwrap_or_else(|| home.clone())
        .join("Claude/claude_desktop_config.json");
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let claude_path = home.join(".config/Claude/claude_desktop_config.json");

    targets.push(HostConfigTarget {
        name: "Claude Desktop",
        path: claude_path,
        server_key: "mcpServers",
        format: HostFormat::CommandArgs,
    });

    // 2. Cursor
    let cursor_global = home.join(".cursor/mcp.json");
    targets.push(HostConfigTarget {
        name: "Cursor (Global)",
        path: cursor_global,
        server_key: "mcpServers",
        format: HostFormat::CommandArgs,
    });

    let cursor_local = PathBuf::from(".cursor/mcp.json");
    targets.push(HostConfigTarget {
        name: "Cursor (Workspace)",
        path: cursor_local,
        server_key: "mcpServers",
        format: HostFormat::CommandArgs,
    });

    // 3. Antigravity
    let agy_global = home.join(".gemini/antigravity-ide/mcp_config.json");
    targets.push(HostConfigTarget {
        name: "Antigravity (Global)",
        path: agy_global,
        server_key: "mcpServers",
        format: HostFormat::CommandArgs,
    });

    let agy_local = PathBuf::from(".agents/mcp_config.json");
    targets.push(HostConfigTarget {
        name: "Antigravity (Workspace)",
        path: agy_local,
        server_key: "mcpServers",
        format: HostFormat::CommandArgs,
    });

    // 4. VS Code
    let vscode_local = PathBuf::from(".vscode/mcp.json");
    targets.push(HostConfigTarget {
        name: "VS Code (Workspace)",
        path: vscode_local,
        server_key: "servers",
        format: HostFormat::CommandArgs,
    });

    targets
}

pub fn configure_agent_target(target: &HostConfigTarget, force: bool) -> anyhow::Result<String> {
    let devflow_entry = json!({
        "command": "devflow",
        "args": ["mcp", "serve"]
    });

    let mut root: Value = if target.path.exists() {
        let content = fs::read_to_string(&target.path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| json!({}))
    } else {
        json!({})
    };

    if !root.is_object() {
        root = json!({});
    }

    let obj = root.as_object_mut().unwrap();
    let servers_val = obj
        .entry(target.server_key.to_string())
        .or_insert_with(|| json!({}));

    if !servers_val.is_object() {
        *servers_val = json!({});
    }

    let servers_map = servers_val.as_object_mut().unwrap();
    if servers_map.contains_key("devflow") && !force {
        return Ok(format!(
            "{} already configured in '{}' (use --force to overwrite)",
            "devflow".cyan(),
            target.path.display()
        ));
    }

    servers_map.insert("devflow".to_string(), devflow_entry);

    if let Some(parent) = target.path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let formatted = serde_json::to_string_pretty(&root)?;
    fs::write(&target.path, formatted)?;

    Ok(format!(
        "Successfully connected DevFlow MCP to {} ({})",
        target.name.bold(),
        target.path.display().to_string().dimmed()
    ))
}
