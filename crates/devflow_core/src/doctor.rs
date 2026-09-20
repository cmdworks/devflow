use devflow_protocol::{DoctorCheck, DoctorReport};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct DoctorEngine;

impl DoctorEngine {
    pub async fn run_diagnostics(project_path: impl AsRef<Path>) -> DoctorReport {
        crate::env::init_environment();
        let path = project_path.as_ref();
        let mut checks = Vec::new();

        // ═══════════════════════════════════════════════════════════════════
        // 1. Apple & Swift Toolchain
        // ═══════════════════════════════════════════════════════════════════
        let apple_category = "Apple & Swift Toolchain";

        #[cfg(target_os = "macos")]
        {
            // 1.1 xcode-select developer directory
            match Command::new("xcode-select").arg("-p").output() {
                Ok(output) if output.status.success() => {
                    let dev_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if dev_path.contains("CommandLineTools") {
                        checks.push(DoctorCheck::warn_with_category(
                            "Xcode Developer Directory",
                            format!("Active path: {} (Command Line Tools)", dev_path),
                            apple_category,
                            Some("sudo xcode-select -s /Applications/Xcode.app/Contents/Developer (if targeting iOS Simulator)".to_string()),
                        ));
                    } else {
                        checks.push(DoctorCheck::pass_with_category(
                            "Xcode Developer Directory",
                            format!("Active path: {}", dev_path),
                            apple_category,
                            Some(dev_path),
                        ));
                    }
                }
                _ => {
                    checks.push(DoctorCheck::fail_with_category(
                        "Xcode Command Line Tools",
                        "Command Line Tools / Developer Directory not configured.",
                        apple_category,
                        Some("xcode-select --install".to_string()),
                    ));
                }
            }

            // 1.2 Swift Compiler
            match Command::new("swift").arg("--version").output() {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let ver = stdout
                        .lines()
                        .next()
                        .unwrap_or("Swift installed")
                        .trim()
                        .to_string();
                    checks.push(DoctorCheck::pass_with_category(
                        "Swift Compiler & Toolchain",
                        ver.clone(),
                        apple_category,
                        Some(ver),
                    ));
                }
                _ => {
                    checks.push(DoctorCheck::fail_with_category(
                        "Swift Compiler",
                        "swift binary not found in system PATH.",
                        apple_category,
                        Some("xcode-select --install".to_string()),
                    ));
                }
            }

            // 1.3 macOS SDK Path
            match Command::new("xcrun")
                .args(["--show-sdk-path", "--sdk", "macosx"])
                .output()
            {
                Ok(output) if output.status.success() => {
                    let sdk_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    checks.push(DoctorCheck::pass_with_category(
                        "macOS SDK",
                        format!("SDK Path: {}", sdk_path),
                        apple_category,
                        Some(sdk_path),
                    ));
                }
                _ => {
                    checks.push(DoctorCheck::warn_with_category(
                        "macOS SDK",
                        "Unable to resolve macOS SDK path via xcrun.",
                        apple_category,
                        Some("xcode-select --install".to_string()),
                    ));
                }
            }

            // 1.4 Xcode License Acceptance & Version
            let xcode_app = Path::new("/Applications/Xcode.app");
            if xcode_app.exists() {
                match Command::new("xcrun")
                    .args(["xcodebuild", "-version"])
                    .output()
                {
                    Ok(output) if output.status.success() => {
                        let ver = String::from_utf8_lossy(&output.stdout)
                            .lines()
                            .next()
                            .unwrap_or("Xcode installed")
                            .trim()
                            .to_string();
                        checks.push(DoctorCheck::pass_with_category(
                            "Xcode.app & License",
                            format!("Installed ({})", ver),
                            apple_category,
                            Some(ver),
                        ));
                    }
                    Ok(output) => {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        if stderr.to_lowercase().contains("license") {
                            checks.push(DoctorCheck::fail_with_category(
                                "Xcode.app License",
                                "Xcode license agreement has not been accepted.",
                                apple_category,
                                Some("sudo xcodebuild -license accept".to_string()),
                            ));
                        } else {
                            checks.push(DoctorCheck::warn_with_category(
                                "Xcode.app",
                                format!("xcodebuild returned error: {}", stderr.trim()),
                                apple_category,
                                Some("sudo xcode-select -s /Applications/Xcode.app/Contents/Developer".to_string()),
                            ));
                        }
                    }
                    _ => {
                        checks.push(DoctorCheck::warn_with_category(
                            "Xcode.app",
                            "Xcode.app found in /Applications but xcodebuild failed to run.",
                            apple_category,
                            Some(
                                "sudo xcode-select -s /Applications/Xcode.app/Contents/Developer"
                                    .to_string(),
                            ),
                        ));
                    }
                }
            } else {
                checks.push(DoctorCheck::warn_with_category(
                    "Xcode.app (iOS Simulator)",
                    "Xcode.app not found in /Applications (Optional for macOS CLI, required for iOS/iPadOS Simulator).",
                    apple_category,
                    Some("Install Xcode from the Mac App Store".to_string()),
                ));
            }

            // 1.5 Apple Simulator Control (simctl)
            match Command::new("xcrun")
                .args(["simctl", "list", "devices", "available"])
                .output()
            {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let count = stdout
                        .lines()
                        .filter(|l| l.contains("Booted") || l.contains("Shutdown"))
                        .count();
                    checks.push(DoctorCheck::pass_with_category(
                        "Apple Simulator Control (simctl)",
                        format!("Available ({} simulator devices discovered)", count),
                        apple_category,
                        None,
                    ));
                }
                _ => {
                    checks.push(DoctorCheck::warn_with_category(
                        "Apple Simulator Control (simctl)",
                        "simctl not operational. Xcode.app is required for simulator control.",
                        apple_category,
                        Some(
                            "sudo xcode-select -s /Applications/Xcode.app/Contents/Developer"
                                .to_string(),
                        ),
                    ));
                }
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            checks.push(DoctorCheck::pass_with_category(
                "Apple & Swift Toolchain",
                "Skipped (non-macOS host platform)",
                apple_category,
                None,
            ));
        }

        // ═══════════════════════════════════════════════════════════════════
        // 2. Android & Kotlin / Java Toolchain
        // ═══════════════════════════════════════════════════════════════════
        let android_category = "Android & Kotlin / Java";

        // 2.1 Java Development Kit (JDK) & JAVA_HOME
        let java_home_env = std::env::var("JAVA_HOME").ok();
        match Command::new("javac").arg("-version").output() {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let ver_str = if !stdout.trim().is_empty() {
                    stdout
                } else {
                    stderr
                };
                let ver = ver_str
                    .lines()
                    .next()
                    .unwrap_or("javac installed")
                    .trim()
                    .to_string();
                let extra = if let Some(ref jh) = java_home_env {
                    format!(" ({}, JAVA_HOME={})", ver, jh)
                } else {
                    format!(" ({})", ver)
                };
                checks.push(DoctorCheck::pass_with_category(
                    "Java Development Kit (JDK)",
                    format!("Operational{}", extra),
                    android_category,
                    Some(ver),
                ));
            }
            _ => {
                if let Some(ref jh) = java_home_env {
                    checks.push(DoctorCheck::warn_with_category(
                        "Java Development Kit (JDK)",
                        format!(
                            "JAVA_HOME is set ({}) but javac is not accessible in PATH.",
                            jh
                        ),
                        android_category,
                        Some("export PATH=$JAVA_HOME/bin:$PATH".to_string()),
                    ));
                } else {
                    checks.push(DoctorCheck::warn_with_category(
                        "Java Development Kit (JDK 17+)",
                        "JDK compiler (javac) not found. Required for Kotlin and Android Gradle builds.",
                        android_category,
                        Some("brew install openjdk@17 && export JAVA_HOME=$(/usr/libexec/java_home -v 17)".to_string()),
                    ));
                }
            }
        }

        // 2.2 Android SDK & ANDROID_HOME
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .ok()
            .map(PathBuf::from);
        let android_sdk_paths = [
            std::env::var("ANDROID_HOME").ok().map(PathBuf::from),
            std::env::var("ANDROID_SDK_ROOT").ok().map(PathBuf::from),
            home_dir.as_ref().map(|h| h.join("Library/Android/sdk")),
            home_dir.as_ref().map(|h| h.join("Android/Sdk")),
        ];

        let mut detected_sdk = None;
        for candidate in android_sdk_paths.into_iter().flatten() {
            if candidate.exists() && candidate.is_dir() {
                detected_sdk = Some(candidate);
                break;
            }
        }

        if let Some(sdk_path) = detected_sdk {
            checks.push(DoctorCheck::pass_with_category(
                "Android SDK Directory",
                format!("Location: {}", sdk_path.display()),
                android_category,
                Some(sdk_path.display().to_string()),
            ));
        } else {
            checks.push(DoctorCheck::warn_with_category(
                "Android SDK Directory",
                "Android SDK directory not found in ANDROID_HOME or standard paths.",
                android_category,
                Some("export ANDROID_HOME=$HOME/Library/Android/sdk".to_string()),
            ));
        }

        // 2.3 Android Debug Bridge (ADB)
        match Command::new("adb").arg("version").output() {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let ver = stdout
                    .lines()
                    .next()
                    .unwrap_or("adb operational")
                    .trim()
                    .to_string();
                checks.push(DoctorCheck::pass_with_category(
                    "Android Debug Bridge (adb)",
                    ver.clone(),
                    android_category,
                    Some(ver),
                ));
            }
            _ => {
                checks.push(DoctorCheck::warn_with_category(
                    "Android Debug Bridge (adb)",
                    "adb command not found in PATH.",
                    android_category,
                    Some("export PATH=$PATH:$ANDROID_HOME/platform-tools".to_string()),
                ));
            }
        }

        // 2.4 Gradle Build Tool
        match Command::new("gradle").arg("--version").output() {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let ver = stdout
                    .lines()
                    .find(|l| l.starts_with("Gradle "))
                    .unwrap_or("Gradle installed")
                    .trim()
                    .to_string();
                checks.push(DoctorCheck::pass_with_category(
                    "Gradle (System)",
                    ver.clone(),
                    android_category,
                    Some(ver),
                ));
            }
            _ => {
                checks.push(DoctorCheck::pass_with_category(
                    "Gradle (System / Wrapper)",
                    "System gradle not in PATH (Projects can use ./gradlew wrapper)",
                    android_category,
                    None,
                ));
            }
        }

        // ═══════════════════════════════════════════════════════════════════
        // 3. Rust & Tauri Toolchain
        // ═══════════════════════════════════════════════════════════════════
        let rust_category = "Rust & Tauri Toolchain";

        // 3.1 Cargo & Rust Compiler
        match Command::new("cargo").arg("--version").output() {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let ver = stdout.trim().to_string();
                checks.push(DoctorCheck::pass_with_category(
                    "Cargo (Rust Toolchain)",
                    ver.clone(),
                    rust_category,
                    Some(ver),
                ));
            }
            _ => {
                checks.push(DoctorCheck::fail_with_category(
                    "Cargo (Rust Toolchain)",
                    "Cargo / rustc not found in PATH.",
                    rust_category,
                    Some(
                        "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
                            .to_string(),
                    ),
                ));
            }
        }

        // 3.2 C Linker / Compiler (cc / clang)
        match Command::new("cc").arg("--version").output() {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let ver = stdout
                    .lines()
                    .next()
                    .unwrap_or("C Linker installed")
                    .trim()
                    .to_string();
                checks.push(DoctorCheck::pass_with_category(
                    "C/C++ Compiler & Linker (cc)",
                    ver.clone(),
                    rust_category,
                    Some(ver),
                ));
            }
            _ => {
                checks.push(DoctorCheck::fail_with_category(
                    "C/C++ Compiler & Linker (cc)",
                    "Native C compiler / linker not found. Required by Rust native crates.",
                    rust_category,
                    Some("xcode-select --install (macOS) or sudo apt install build-essential (Linux)".to_string()),
                ));
            }
        }

        // 3.3 pkg-config tool
        match Command::new("pkg-config").arg("--version").output() {
            Ok(output) if output.status.success() => {
                let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
                checks.push(DoctorCheck::pass_with_category(
                    "pkg-config Tool",
                    format!("Installed (v{})", ver),
                    rust_category,
                    Some(ver),
                ));
            }
            _ => {
                checks.push(DoctorCheck::warn_with_category(
                    "pkg-config Tool",
                    "pkg-config not found. Some native Rust dependencies may fail to build.",
                    rust_category,
                    Some("brew install pkg-config".to_string()),
                ));
            }
        }

        // 3.4 Tauri CLI
        match Command::new("cargo").args(["tauri", "--version"]).output() {
            Ok(output) if output.status.success() => {
                let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
                checks.push(DoctorCheck::pass_with_category(
                    "Tauri CLI (cargo-tauri)",
                    ver.clone(),
                    rust_category,
                    Some(ver),
                ));
            }
            _ => {
                checks.push(DoctorCheck::pass_with_category(
                    "Tauri CLI",
                    "cargo-tauri not installed globally (Projects can run via npx @tauri-apps/cli)",
                    rust_category,
                    None,
                ));
            }
        }

        // ═══════════════════════════════════════════════════════════════════
        // 4. Core Web & Scripting Runtimes
        // ═══════════════════════════════════════════════════════════════════
        let web_category = "Core Web & Scripting Runtimes";

        // 4.1 Node.js
        checks.push(Self::check_tool_with_category(
            "node",
            &["--version"],
            "Node.js Runtime",
            web_category,
            false,
        ));

        // 4.2 Bun
        checks.push(Self::check_tool_with_category(
            "bun",
            &["--version"],
            "Bun Runtime",
            web_category,
            false,
        ));

        // 4.3 Flutter
        checks.push(Self::check_tool_with_category(
            "flutter",
            &["--version"],
            "Flutter SDK",
            web_category,
            false,
        ));

        // ═══════════════════════════════════════════════════════════════════
        // 5. Workspace & Project Diagnostics
        // ═══════════════════════════════════════════════════════════════════
        let ws_category = "Workspace & Project Setup";

        // 5.1 devflow.toml
        if path.join("devflow.toml").exists() {
            match crate::config::DevflowConfig::load_from_file(path.join("devflow.toml")) {
                Ok(cfg) => {
                    checks.push(DoctorCheck::pass_with_category(
                        "devflow.toml Configuration",
                        format!(
                            "Valid configuration (platform: {}, framework: {})",
                            cfg.project.platform, cfg.project.framework
                        ),
                        ws_category,
                        None,
                    ));
                }
                Err(e) => {
                    checks.push(DoctorCheck::fail_with_category(
                        "devflow.toml Configuration",
                        format!("Invalid configuration: {}", e),
                        ws_category,
                        Some("Check devflow.toml syntax and schema.".to_string()),
                    ));
                }
            }
        } else {
            checks.push(DoctorCheck::warn_with_category(
                "devflow.toml Configuration",
                "No devflow.toml found in project root (DevFlow auto-detection active).",
                ws_category,
                Some("Run 'devflow init' to scaffold custom project settings.".to_string()),
            ));
        }

        // 5.2 gradlew wrapper executable check
        let gradlew_candidates = [path.join("gradlew"), path.join("android/gradlew")];
        for gw in &gradlew_candidates {
            if gw.exists() {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(meta) = std::fs::metadata(gw) {
                        let is_executable = (meta.permissions().mode() & 0o111) != 0;
                        if is_executable {
                            checks.push(DoctorCheck::pass_with_category(
                                "Gradle Wrapper Permissions",
                                format!(
                                    "Executable permissions verified for {}",
                                    gw.file_name().unwrap_or_default().to_string_lossy()
                                ),
                                ws_category,
                                None,
                            ));
                        } else {
                            checks.push(DoctorCheck::fail_with_category(
                                "Gradle Wrapper Permissions",
                                format!("{} lacks execute (+x) permissions.", gw.display()),
                                ws_category,
                                Some(format!("chmod +x {}", gw.display())),
                            ));
                        }
                    }
                }
                break;
            }
        }

        // 5.3 Swift Package Manager Package.swift check
        let package_swift = path.join("Package.swift");
        if package_swift.exists() {
            checks.push(DoctorCheck::pass_with_category(
                "Swift Package Manifest",
                "Package.swift detected in workspace root.",
                ws_category,
                None,
            ));
        }

        DoctorReport::new(path.to_string_lossy().to_string(), checks)
    }

    fn check_tool_with_category(
        bin: &str,
        args: &[&str],
        display_name: &str,
        category: &str,
        required: bool,
    ) -> DoctorCheck {
        match Command::new(bin).args(args).output() {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let first_line = stdout
                    .lines()
                    .next()
                    .unwrap_or("Installed")
                    .trim()
                    .to_string();
                DoctorCheck::pass_with_category(
                    display_name,
                    first_line.clone(),
                    category,
                    Some(first_line),
                )
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if required {
                    DoctorCheck::fail_with_category(
                        display_name,
                        format!("Tool '{}' failed: {}", bin, stderr.trim()),
                        category,
                        Some(format!(
                            "Ensure '{}' is properly installed and accessible in PATH.",
                            bin
                        )),
                    )
                } else {
                    DoctorCheck::warn_with_category(
                        display_name,
                        format!("Tool '{}' returned non-zero exit code.", bin),
                        category,
                        Some(format!(
                            "Optional for generic projects, required for native {} builds.",
                            bin
                        )),
                    )
                }
            }
            Err(_) => {
                if required {
                    DoctorCheck::fail_with_category(
                        display_name,
                        format!("'{}' not found in PATH", bin),
                        category,
                        Some(format!(
                            "Please install '{}' and add it to your system PATH.",
                            bin
                        )),
                    )
                } else {
                    DoctorCheck::warn_with_category(
                        display_name,
                        format!("'{}' not found (optional)", bin),
                        category,
                        Some(format!("Install '{}' if building {} projects.", bin, bin)),
                    )
                }
            }
        }
    }
}
