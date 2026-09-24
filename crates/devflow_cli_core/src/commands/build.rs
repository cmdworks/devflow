use colored::*;
use devflow_core::project::Project;
use devflow_devices::DeviceManager;
use devflow_frameworks::adapter::BuildContext;
use devflow_frameworks::registry::FrameworkRegistry;

pub async fn handle_build(release: bool, target: Option<String>, json: bool) -> anyhow::Result<()> {
    let current_dir = std::env::current_dir()?;
    let project = Project::detect(&current_dir)?;
    let registry = FrameworkRegistry::new();
    let adapter = registry.select_adapter(&project);

    let device =
        DeviceManager::find_best_match(Some(project.detected_platform), target.as_deref()).await;

    let ctx = BuildContext {
        project_dir: project.root_dir.clone(),
        config: project.effective_config(),
        target_device: device,
        is_release: release,
    };

    println!(
        "{} Building project '{}' using {} adapter...",
        "🔨".cyan(),
        project.name.bold(),
        adapter.name().magenta()
    );
    let res = adapter.build(&ctx).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&res)?);
        return Ok(());
    }

    if res.success {
        println!(
            "{} Build succeeded in {}ms",
            "✓".green().bold(),
            res.duration_ms
        );
        if let Some(ref art) = res.artifact {
            println!("  {} {}", "Artifact:".cyan(), art.path.bold());
        }
    } else {
        println!("{} Build failed in {}ms", "✗".red().bold(), res.duration_ms);
        if let Some(ref err) = res.error_message {
            println!("{}\n", err.red());
        }
    }

    Ok(())
}
