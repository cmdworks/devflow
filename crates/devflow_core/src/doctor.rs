use devflow_protocol::{DoctorCheck, DoctorReport};
use std::path::Path;
use std::process::Command;

pub struct DoctorEngine;

impl DoctorEngine {
    pub async fn run_diagnostics(project_path: impl AsRef<Path>) -> DoctorReport {
        crate::env::init_environment();
        let path = project_path.as_ref();
        let mut checks = Vec::new();

        // 1. Rust / Cargo
        checks.push(Self::check_tool("cargo", &["--version"], "Cargo (Rust toolchain)", true));

        // 2. Android Debug Bridge (ADB)
        checks.push(Self::check_tool(
            "adb",
            &["version"],
            "Android Debug Bridge (adb)",
            false,
        ));

        // 3. Java Runtime & Compiler
        checks.push(Self::check_tool(
            "javac",
            &["-version"],
            "Java Development Kit (JDK)",
            false,
        ));

        // 4. Apple Tooling (xcrun / simctl)
        checks.push(Self::check_tool(
            "xcrun",
            &["simctl", "help"],
            "Apple Simulator Control (simctl)",
            false,
        ));

        // 5. Swift Compiler
        checks.push(Self::check_tool(
            "swift",
            &["--version"],
            "Swift Compiler & Package Manager",
            false,
        ));

        // 6. Gradle
        checks.push(Self::check_tool(
            "gradle",
            &["--version"],
            "Gradle Build Tool",
            false,
        ));

        // 7. Node.js
        checks.push(Self::check_tool(
            "node",
            &["--version"],
            "Node.js Runtime",
            false,
        ));

        // 8. Bun
        checks.push(Self::check_tool(
            "bun",
            &["--version"],
            "Bun Runtime",
            false,
        ));

        // 9. Flutter
        checks.push(Self::check_tool(
            "flutter",
            &["--version"],
            "Flutter SDK",
            false,
        ));

        // 8. Project Configuration Check
        if path.join("devflow.toml").exists() {
            match crate::config::DevflowConfig::load_from_file(path.join("devflow.toml")) {
                Ok(cfg) => {
                    checks.push(DoctorCheck::pass(
                        "devflow.toml",
                        format!("Valid configuration (platform: {}, framework: {})", cfg.project.platform, cfg.project.framework),
                        None,
                    ));
                }
                Err(e) => {
                    checks.push(DoctorCheck::fail(
                        "devflow.toml",
                        format!("Invalid configuration: {}", e),
                        Some("Check devflow.toml syntax and schema.".to_string()),
                    ));
                }
            }
        } else {
            checks.push(DoctorCheck::warn(
                "devflow.toml",
                "No devflow.toml found in project root (will use auto-detection).",
                Some("Run 'devflow init' to scaffold a project configuration.".to_string()),
            ));
        }

        DoctorReport::new(path.to_string_lossy().to_string(), checks)
    }

    fn check_tool(bin: &str, args: &[&str], display_name: &str, required: bool) -> DoctorCheck {
        match Command::new(bin).args(args).output() {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let first_line = stdout.lines().next().unwrap_or("Installed").trim().to_string();
                DoctorCheck::pass(display_name, first_line.clone(), Some(first_line))
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if required {
                    DoctorCheck::fail(
                        display_name,
                        format!("Tool '{}' failed: {}", bin, stderr.trim()),
                        Some(format!("Ensure '{}' is properly installed and accessible in PATH.", bin)),
                    )
                } else {
                    DoctorCheck::warn(
                        display_name,
                        format!("Tool '{}' returned non-zero exit code.", bin),
                        Some(format!("Optional for generic projects, required for native {} builds.", bin)),
                    )
                }
            }
            Err(_) => {
                if required {
                    DoctorCheck::fail(
                        display_name,
                        format!("'{}' not found in PATH", bin),
                        Some(format!("Please install '{}' and add it to your system PATH.", bin)),
                    )
                } else {
                    DoctorCheck::warn(
                        display_name,
                        format!("'{}' not found (optional)", bin),
                        Some(format!("Install '{}' if building {} projects.", bin, bin)),
                    )
                }
            }
        }
    }
}
