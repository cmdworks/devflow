use anyhow::{Context, Result};
use clap::CommandFactory;
use clap_complete::{generate, Shell};
use colored::*;
use std::path::PathBuf;

pub fn install_cli_symlink(custom_target_dir: Option<PathBuf>) -> Result<String> {
    let current_exe = std::env::current_exe()
        .context("Failed to determine current executable path")?
        .canonicalize()
        .unwrap_or_else(|_| std::env::current_exe().unwrap());

    // Target directories in order of preference
    let candidate_dirs = if let Some(dir) = custom_target_dir {
        vec![dir]
    } else {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        vec![
            PathBuf::from("/usr/local/bin"),
            PathBuf::from(format!("{}/.local/bin", home)),
            PathBuf::from(format!("{}/.cargo/bin", home)),
            PathBuf::from(format!("{}/bin", home)),
        ]
    };

    let mut installed_path = None;
    let mut last_error = None;

    for dir in &candidate_dirs {
        if !dir.exists() {
            let _ = std::fs::create_dir_all(dir);
        }

        let link_path = dir.join("devflow");

        // Remove existing link/file if present
        if link_path.exists() || link_path.is_symlink() {
            let _ = std::fs::remove_file(&link_path);
        }

        #[cfg(unix)]
        {
            match std::os::unix::fs::symlink(&current_exe, &link_path) {
                Ok(_) => {
                    installed_path = Some(link_path);
                    break;
                }
                Err(e) => {
                    last_error = Some(format!("{}: {}", link_path.display(), e));
                }
            }
        }

        #[cfg(windows)]
        {
            match std::os::windows::fs::symlink_file(&current_exe, &link_path) {
                Ok(_) => {
                    installed_path = Some(link_path);
                    break;
                }
                Err(e) => {
                    last_error = Some(format!("{}: {}", link_path.display(), e));
                }
            }
        }
    }

    if let Some(target) = installed_path {
        Ok(format!(
            "Successfully created 'devflow' command symlink: {} -> {}",
            target.display().to_string().cyan().bold(),
            current_exe.display().to_string().dimmed()
        ))
    } else {
        anyhow::bail!(
            "Failed to install 'devflow' symlink into system PATH. (Tried: {}). Error: {:?}",
            candidate_dirs
                .iter()
                .map(|d| d.display().to_string())
                .collect::<Vec<_>>()
                .join(", "),
            last_error
        );
    }
}

pub fn uninstall_cli_symlink() -> Result<String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let candidate_dirs = vec![
        PathBuf::from("/usr/local/bin"),
        PathBuf::from(format!("{}/.local/bin", home)),
        PathBuf::from(format!("{}/.cargo/bin", home)),
        PathBuf::from(format!("{}/bin", home)),
    ];

    let mut removed = Vec::new();
    for dir in candidate_dirs {
        let link_path = dir.join("devflow");
        if (link_path.exists() || link_path.is_symlink())
            && std::fs::remove_file(&link_path).is_ok()
        {
            removed.push(link_path.display().to_string());
        }
    }

    if removed.is_empty() {
        Ok("No existing 'devflow' symlink found in candidate PATH directories.".to_string())
    } else {
        Ok(format!(
            "Removed 'devflow' symlink from: {}",
            removed.join(", ")
        ))
    }
}

pub fn generate_shell_hook(shell_name: &str) -> String {
    match shell_name.to_lowercase().as_str() {
        "fish" => r#"# DevFlow Shell Integration for Fish
alias dfr="devflow reload"
alias dfrs="devflow restart"
alias dflog="devflow logs -f"
alias dfopen="devflow open"
alias dfdoc="devflow doctor"

function __devflow_active_session
    if test -n "$DEVFLOW_SESSION"
        echo "[devflow:$DEVFLOW_SESSION]"
    end
end
"#
        .to_string(),

        "bash" => r#"# DevFlow Shell Integration for Bash
alias dfr='devflow reload'
alias dfrs='devflow restart'
alias dflog='devflow logs -f'
alias dfopen='devflow open'
alias dfdoc='devflow doctor'

# Export DevFlow active terminal ID if inside VS Code
if [ -n "$VSCODE_GIT_ASKPASS_NODE" ] || [ -n "$VSCODE_INJECTION" ]; then
    export DEVFLOW_TERMINAL_SOURCE="vscode"
fi
"#
        .to_string(),

        _ => r#"# DevFlow Shell Integration for Zsh / Default
alias dfr='devflow reload'
alias dfrs='devflow restart'
alias dflog='devflow logs -f'
alias dfopen='devflow open'
alias dfdoc='devflow doctor'

# Fast DevFlow shell wrapper
devflow_reload_widget() {
    devflow reload
    zle reset-prompt
}

# Auto-detect VS Code integrated terminal environment
if [[ -n "$VSCODE_GIT_ASKPASS_NODE" || -n "$VSCODE_INJECTION" ]]; then
    export DEVFLOW_TERMINAL_SOURCE="vscode"
fi
"#
        .to_string(),
    }
}

pub fn generate_completions<C: CommandFactory>(shell_name: &str) -> Result<String> {
    let shell = match shell_name.to_lowercase().as_str() {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        "powershell" | "pwsh" => Shell::PowerShell,
        "elvish" => Shell::Elvish,
        _ => anyhow::bail!(
            "Unsupported shell '{}'. Supported: bash, zsh, fish, powershell, elvish",
            shell_name
        ),
    };

    let mut cmd = C::command();
    let mut buf = Vec::new();
    generate(shell, &mut cmd, "devflow", &mut buf);
    String::from_utf8(buf).context("Failed to format completion output as UTF-8 string")
}
