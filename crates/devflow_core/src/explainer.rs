use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorExplanation {
    pub framework: String,
    pub title: String,
    pub issue: String,
    pub fix_hint: String,
    pub steps: Vec<String>,
    pub ansi_banner: String,
}

pub struct ToolchainExplainer;

impl ToolchainExplainer {
    pub fn explain_error(framework: &str, stderr: &str, stdout: &str, target_name: &str) -> Option<ErrorExplanation> {
        let combined = format!("{}\n{}", stderr, stdout).to_lowercase();
        let fw = framework.to_lowercase();

        // 1. Swift & Apple Toolchain Errors
        if fw.contains("swift") || fw.contains("xcode") || fw.contains("apple") || combined.contains("swift") || combined.contains("xcode") {
            if combined.contains("tool 'xcodebuild' requires xcode") || combined.contains("active developer directory '/library/developer/commandlinetools'") {
                return Some(Self::build_explanation(
                    "swift",
                    target_name,
                    "Xcode Command Line Tools Active (Full Xcode.app Required)",
                    "Active developer directory is set to Command Line Tools, but Xcode.app is required for simulator/iOS builds.",
                    "sudo xcode-select -s /Applications/Xcode.app/Contents/Developer",
                    vec![
                        "Check if Xcode is installed in /Applications: ls /Applications/Xcode.app".to_string(),
                        "Point xcode-select to full Xcode: sudo xcode-select -s /Applications/Xcode.app/Contents/Developer".to_string(),
                        "Accept the Xcode license agreement: sudo xcodebuild -license accept".to_string(),
                    ],
                ));
            }

            if combined.contains("license agreement") || combined.contains("agree to the xcode") {
                return Some(Self::build_explanation(
                    "swift",
                    target_name,
                    "Xcode License Agreement Not Accepted",
                    "Xcode license agreement must be accepted before compiler tools can run.",
                    "sudo xcodebuild -license accept",
                    vec![
                        "Run: sudo xcodebuild -license accept".to_string(),
                    ],
                ));
            }

            if combined.contains("unable to find sdk 'macosx'") || combined.contains("active developer path") || combined.contains("xcode-select: error") {
                return Some(Self::build_explanation(
                    "swift",
                    target_name,
                    "Apple Command Line Tools or SDK Missing",
                    "macOS SDK / Command Line Tools are missing or pointing to an invalid directory.",
                    "xcode-select --install",
                    vec![
                        "Install Command Line Tools: xcode-select --install".to_string(),
                        "Or reset developer directory to default: sudo xcode-select -r".to_string(),
                    ],
                ));
            }

            if combined.contains("cannot find module") || combined.contains("no such module") || combined.contains("missing package dependency") {
                return Some(Self::build_explanation(
                    "swift",
                    target_name,
                    "Missing Swift Package Dependencies",
                    "Swift Package Manager (SPM) dependencies are unresolved or missing.",
                    "swift package resolve",
                    vec![
                        "Fetch dependencies: swift package resolve".to_string(),
                        "Update package dependencies: swift package update".to_string(),
                        "Clean build cache: rm -rf .build/".to_string(),
                    ],
                ));
            }

            if combined.contains("database is locked") || combined.contains("invalid build database") || combined.contains("corrupt build database") {
                return Some(Self::build_explanation(
                    "swift",
                    target_name,
                    "SwiftPM Build Cache Locked/Corrupt",
                    "Swift package build cache is locked or in an inconsistent state.",
                    "rm -rf .build/",
                    vec![
                        "Remove build cache: rm -rf .build/".to_string(),
                        "Re-run build: swift build".to_string(),
                    ],
                ));
            }
        }

        // 2. Kotlin / Android & Gradle Errors
        if fw.contains("kotlin") || fw.contains("android") || fw.contains("gradle") || combined.contains("gradle") || combined.contains("android") {
            if combined.contains("java_home is not set") || combined.contains("java: command not found") || combined.contains("could not find or load main class") {
                return Some(Self::build_explanation(
                    "kotlin",
                    target_name,
                    "JDK Not Found / JAVA_HOME Unset",
                    "Java Development Kit (JDK 17+ recommended) is not installed or JAVA_HOME is not set.",
                    "brew install openjdk@17 && export JAVA_HOME=$(/usr/libexec/java_home -v 17)",
                    vec![
                        "Install OpenJDK: brew install openjdk@17".to_string(),
                        "Set JAVA_HOME in ~/.zshrc: export JAVA_HOME=$(/usr/libexec/java_home -v 17)".to_string(),
                        "Add to PATH: export PATH=$JAVA_HOME/bin:$PATH".to_string(),
                    ],
                ));
            }

            if combined.contains("unsupported class file major version") || combined.contains("incompatible with this version of java") {
                return Some(Self::build_explanation(
                    "kotlin",
                    target_name,
                    "Java / Gradle Version Incompatibility",
                    "The current Java version is incompatible with this project's Gradle version (JDK 17 recommended).",
                    "export JAVA_HOME=$(/usr/libexec/java_home -v 17)",
                    vec![
                        "Switch to JDK 17: export JAVA_HOME=$(/usr/libexec/java_home -v 17)".to_string(),
                        "Or upgrade Gradle wrapper: ./gradlew wrapper --gradle-version 8.7".to_string(),
                    ],
                ));
            }

            if combined.contains("sdk location not found") || combined.contains("android_home") || combined.contains("sdk.dir") {
                return Some(Self::build_explanation(
                    "kotlin",
                    target_name,
                    "Android SDK Not Found",
                    "Android SDK directory location is missing or ANDROID_HOME is unset.",
                    "export ANDROID_HOME=$HOME/Library/Android/sdk",
                    vec![
                        "Export ANDROID_HOME: export ANDROID_HOME=$HOME/Library/Android/sdk".to_string(),
                        "Add platform-tools to PATH: export PATH=$PATH:$ANDROID_HOME/platform-tools:$ANDROID_HOME/cmdline-tools/latest/bin".to_string(),
                        "Or set in project local.properties: echo \"sdk.dir=$HOME/Library/Android/sdk\" > local.properties".to_string(),
                    ],
                ));
            }

            if combined.contains("failed to find target with hash string 'android-") || combined.contains("platforms;android-") {
                return Some(Self::build_explanation(
                    "kotlin",
                    target_name,
                    "Missing Android Platform SDK",
                    "The requested Android target platform API version is not installed in the SDK.",
                    "sdkmanager --install \"platforms;android-34\"",
                    vec![
                        "Install platform: sdkmanager --install \"platforms;android-34\"".to_string(),
                        "Install build-tools: sdkmanager --install \"build-tools;34.0.0\"".to_string(),
                    ],
                ));
            }

            if combined.contains("permission denied") && combined.contains("gradlew") {
                return Some(Self::build_explanation(
                    "kotlin",
                    target_name,
                    "Gradle Wrapper Not Executable",
                    "The gradlew shell script lacks execute (+x) permissions.",
                    "chmod +x gradlew",
                    vec![
                        "Grant execute permission: chmod +x gradlew".to_string(),
                    ],
                ));
            }

            if combined.contains("could not resolve all dependencies") || combined.contains("connection refused") || combined.contains("timed out") {
                return Some(Self::build_explanation(
                    "kotlin",
                    target_name,
                    "Gradle Dependency Resolution Failed",
                    "Network error or missing remote repository while downloading dependencies.",
                    "./gradlew assembleDebug --refresh-dependencies",
                    vec![
                        "Refresh dependencies: ./gradlew assembleDebug --refresh-dependencies".to_string(),
                        "Verify network connection to maven.google.com and repo.maven.apache.org".to_string(),
                    ],
                ));
            }
        }

        // 3. Rust & Cargo Errors
        if fw.contains("cargo") || fw.contains("rust") || combined.contains("cargo") || combined.contains("rustc") {
            if combined.contains("linker `cc` not found") || combined.contains("error: linker") || combined.contains("clang: error") {
                return Some(Self::build_explanation(
                    "cargo",
                    target_name,
                    "C/C++ Linker Not Found",
                    "The Rust compiler cannot locate a C compiler/linker (cc/clang).",
                    "xcode-select --install",
                    vec![
                        "Install Command Line Tools (macOS): xcode-select --install".to_string(),
                        "Or install GCC (Linux): sudo apt install build-essential".to_string(),
                    ],
                ));
            }

            if combined.contains("openssl") && (combined.contains("could not find directory") || combined.contains("openssl-sys")) {
                return Some(Self::build_explanation(
                    "cargo",
                    target_name,
                    "OpenSSL Development Headers Missing",
                    "Crates requiring OpenSSL cannot locate the development headers/libraries.",
                    "brew install openssl@3 && export OPENSSL_DIR=$(brew --prefix openssl@3)",
                    vec![
                        "Install OpenSSL: brew install openssl@3".to_string(),
                        "Export OPENSSL_DIR: export OPENSSL_DIR=$(brew --prefix openssl@3)".to_string(),
                        "Export PKG_CONFIG_PATH: export PKG_CONFIG_PATH=\"$(brew --prefix openssl@3)/lib/pkgconfig:$PKG_CONFIG_PATH\"".to_string(),
                    ],
                ));
            }

            if combined.contains("pkg-config") || combined.contains("pkg_config") {
                return Some(Self::build_explanation(
                    "cargo",
                    target_name,
                    "pkg-config Tool Missing",
                    "Native system dependency discovery via pkg-config failed.",
                    "brew install pkg-config",
                    vec![
                        "Install pkg-config: brew install pkg-config".to_string(),
                    ],
                ));
            }
        }

        // 4. Tauri Framework Errors
        if fw.contains("tauri") || combined.contains("tauri") {
            if combined.contains("tauri: command not found") || combined.contains("cannot find module '@tauri-apps/cli'") {
                return Some(Self::build_explanation(
                    "tauri",
                    target_name,
                    "Tauri CLI Missing",
                    "Tauri CLI package is not installed in the project dependencies.",
                    "npm install -D @tauri-apps/cli",
                    vec![
                        "Install Tauri CLI: npm install -D @tauri-apps/cli".to_string(),
                    ],
                ));
            }

            if combined.contains("failed to build frontend") || combined.contains("vite: not found") {
                return Some(Self::build_explanation(
                    "tauri",
                    target_name,
                    "Tauri Frontend Build Failed",
                    "The frontend build command failed. Dependencies may need installation.",
                    "npm install && npm run build",
                    vec![
                        "Install node modules: npm install".to_string(),
                        "Run frontend build: npm run build".to_string(),
                    ],
                ));
            }
        }

        None
    }

    fn build_explanation(
        framework: &str,
        target_name: &str,
        title: &str,
        issue: &str,
        fix_hint: &str,
        steps: Vec<String>,
    ) -> ErrorExplanation {
        let mut banner = String::new();
        banner.push_str("\r\n\x1b[31;1m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m\r\n");
        banner.push_str(&format!("\x1b[31;1m ✗ BUILD FAILED:\x1b[0m \x1b[1;37m{} [{}]\x1b[0m\r\n", target_name, framework));
        banner.push_str(&format!("\x1b[33;1m 💡 Issue Detected:\x1b[0m \x1b[38;5;222m{}\x1b[0m\r\n", title));
        banner.push_str(&format!("    \x1b[90m{}\x1b[0m\r\n", issue));
        banner.push_str("\x1b[36;1m 🔧 How to Fix:\x1b[0m\r\n");
        for (i, step) in steps.iter().enumerate() {
            banner.push_str(&format!("    \x1b[36m{}.\x1b[0m \x1b[97m{}\x1b[0m\r\n", i + 1, step));
        }
        banner.push_str(&format!("\x1b[32;1m 📋 Quick Command:\x1b[0m \x1b[40;1;32m {} \x1b[0m\r\n", fix_hint));
        banner.push_str("\x1b[31;1m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m\r\n");

        ErrorExplanation {
            framework: framework.to_string(),
            title: title.to_string(),
            issue: issue.to_string(),
            fix_hint: fix_hint.to_string(),
            steps,
            ansi_banner: banner,
        }
    }
}
