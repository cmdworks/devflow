use std::path::{Path, PathBuf};
use tracing::debug;

/// Automatically detect and configure SDKs, runtimes, and toolchain PATHs
/// into the current process environment so all subcommands and build tools work out of the box.
pub fn init_environment() {
    let home = std::env::var("HOME").ok().map(PathBuf::from);

    // 1. Android SDK detection
    if std::env::var("ANDROID_HOME").is_err() {
        if let Some(ref h) = home {
            let candidates = [
                h.join("Library/Android/sdk"),
                h.join("Android/Sdk"),
                h.join(".android-sdk"),
            ];
            for candidate in &candidates {
                if candidate.exists() {
                    let p = candidate.display().to_string();
                    std::env::set_var("ANDROID_HOME", &p);
                    std::env::set_var("ANDROID_SDK_ROOT", &p);
                    debug!("Auto-configured ANDROID_HOME={}", p);
                    break;
                }
            }
        }
    }

    // 2. Java Home detection (prefer JDK 21 for Android Gradle compatibility)
    if std::env::var("JAVA_HOME").is_err() {
        let java_candidates = [
            PathBuf::from("/Library/Java/JavaVirtualMachines/jdk-21.jdk/Contents/Home"),
            PathBuf::from("/Library/Java/JavaVirtualMachines/jdk-27.jdk/Contents/Home"),
            PathBuf::from("/Library/Java/JavaVirtualMachines/temurin-21.jdk/Contents/Home"),
            PathBuf::from("/Library/Java/JavaVirtualMachines/zulu-21.jdk/Contents/Home"),
            PathBuf::from("/Library/Java/JavaVirtualMachines/openjdk-21.jdk/Contents/Home"),
        ];

        let mut found_java = false;
        for j in &java_candidates {
            if j.exists() {
                std::env::set_var("JAVA_HOME", j.display().to_string());
                debug!("Auto-configured JAVA_HOME={}", j.display());
                found_java = true;
                break;
            }
        }

        if !found_java {
            // Try macOS /usr/libexec/java_home
            if let Ok(output) = std::process::Command::new("/usr/libexec/java_home").output() {
                if output.status.success() {
                    let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path_str.is_empty() && Path::new(&path_str).exists() {
                        std::env::set_var("JAVA_HOME", &path_str);
                        debug!("Auto-configured JAVA_HOME via /usr/libexec/java_home={}", path_str);
                    }
                }
            }
        }
    }

    // 3. Smart PATH enrichment
    let mut extra_paths = Vec::new();

    if let Some(ref h) = home {
        // Local binaries
        let local_bin = h.join(".local/bin");
        if local_bin.exists() {
            extra_paths.push(local_bin);
        }

        // Rust / Cargo
        let cargo_bin = h.join(".cargo/bin");
        if cargo_bin.exists() {
            extra_paths.push(cargo_bin);
        }

        // Bun
        let bun_bin = h.join(".bun/bin");
        if bun_bin.exists() {
            extra_paths.push(bun_bin);
        }

        // NVM / Node versions
        let nvm_node_dir = h.join(".nvm/versions/node");
        if nvm_node_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&nvm_node_dir) {
                let mut versions: Vec<PathBuf> = entries
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| p.is_dir())
                    .collect();
                versions.sort_by(|a, b| b.cmp(a)); // latest version first
                for v in versions {
                    let bin = v.join("bin");
                    if bin.exists() {
                        extra_paths.push(bin);
                        break;
                    }
                }
            }
        }
    }

    // Android platform-tools & emulator
    if let Ok(android_home) = std::env::var("ANDROID_HOME") {
        let ah = PathBuf::from(android_home);
        let pt = ah.join("platform-tools");
        if pt.exists() {
            extra_paths.push(pt);
        }
        let emu = ah.join("emulator");
        if emu.exists() {
            extra_paths.push(emu);
        }
        let cmdline = ah.join("cmdline-tools/latest/bin");
        if cmdline.exists() {
            extra_paths.push(cmdline);
        }
    }

    // Java bin
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        let jb = PathBuf::from(java_home).join("bin");
        if jb.exists() {
            extra_paths.push(jb);
        }
    }

    // Prepend to PATH if not already present
    let current_path = std::env::var("PATH").unwrap_or_default();
    let path_entries: Vec<PathBuf> = std::env::split_paths(&current_path).collect();

    let mut new_paths = Vec::new();
    for p in extra_paths {
        if !path_entries.contains(&p) {
            new_paths.push(p);
        }
    }

    if !new_paths.is_empty() {
        new_paths.extend(path_entries);
        if let Ok(joined) = std::env::join_paths(new_paths) {
            std::env::set_var("PATH", joined);
        }
    }
}
