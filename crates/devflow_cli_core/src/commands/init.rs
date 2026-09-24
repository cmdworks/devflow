use colored::*;
use devflow_core::config::DevflowConfig;

pub async fn handle_init(
    _platform: Option<String>,
    framework: Option<String>,
) -> anyhow::Result<()> {
    let current_dir = std::env::current_dir()?;
    let config_path = current_dir.join("devflow.toml");

    if config_path.exists() {
        println!(
            "{}",
            "devflow.toml already exists in current directory.".yellow()
        );
        return Ok(());
    }

    let project_name = current_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "my-app".to_string());

    let fw_str = framework
        .unwrap_or_else(|| "generic".to_string())
        .to_lowercase();
    let content = match fw_str.as_str() {
        "kotlin" | "android" => DevflowConfig::template_android(&project_name),
        "swift" | "swiftpm" | "macos" => DevflowConfig::template_swift(&project_name),
        "react-native" | "rn" => DevflowConfig::template_react_native(&project_name),
        "flutter" => DevflowConfig::template_flutter(&project_name),
        "tauri" => DevflowConfig::template_tauri(&project_name),
        _ => DevflowConfig::template_generic(&project_name),
    };

    std::fs::write(&config_path, content)?;
    println!(
        "{} Created {}",
        "✓".green().bold(),
        "devflow.toml".cyan().bold()
    );
    println!("Edit devflow.toml to customize build, install, launch, and watch settings.");
    Ok(())
}
