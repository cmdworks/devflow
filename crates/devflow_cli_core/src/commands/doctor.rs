use colored::*;
use devflow_core::doctor::DoctorEngine;
use devflow_protocol::CheckStatus;
use std::path::PathBuf;

pub async fn handle_doctor(project_path: PathBuf, json: bool) -> anyhow::Result<()> {
    let report = DoctorEngine::run_diagnostics(&project_path).await;

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!("\n{}", "═══ DevFlow Doctor Diagnostics ═══".cyan().bold());
    println!("Project path: {}\n", report.project_path.dimmed());

    for check in &report.checks {
        let (icon, status_str) = match check.status {
            CheckStatus::Passed => ("✓".green().bold(), "PASS".green()),
            CheckStatus::Warning => ("!".yellow().bold(), "WARN".yellow()),
            CheckStatus::Failed => ("✗".red().bold(), "FAIL".red()),
            CheckStatus::Skipped => ("-".dimmed(), "SKIP".dimmed()),
        };

        println!(
            " {} [{}] {} — {}",
            icon,
            status_str,
            check.name.bold(),
            check.message
        );
        if let Some(ref hint) = check.fix_hint {
            println!("     {} {}", "Fix hint:".magenta(), hint.dimmed());
        }
    }

    println!(
        "\nSummary: {} passed, {} warnings, {} failed",
        report.passed_count.to_string().green(),
        report.warning_count.to_string().yellow(),
        report.failure_count.to_string().red()
    );

    if report.is_healthy() {
        println!(
            "{}\n",
            "✓ Your development environment is ready!".green().bold()
        );
    } else {
        println!(
            "{}\n",
            "! Some required tools or configs are missing."
                .yellow()
                .bold()
        );
    }

    Ok(())
}
