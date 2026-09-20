fn main() {
    println!("cargo:rerun-if-changed=native/devflow-dialog-macos.m");
    println!("cargo:rerun-if-changed=native/devflow-macos.m");
    println!("cargo:rerun-if-changed=build.rs");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    if target_os == "macos" {
        // Compile helper static library for setting macOS dock icon
        cc::Build::new()
            .file("native/devflow-macos.m")
            .flag("-fobjc-arc")
            .compile("devflow_macos");
        println!("cargo:rustc-link-lib=framework=Cocoa");

        let native_src = "native/devflow-dialog-macos.m";
        if std::path::Path::new(native_src).exists() {
            if let Ok(out_dir) = std::env::var("OUT_DIR") {
                let out_bin = std::path::Path::new(&out_dir).join("devflow-dialog-macos");
                let should_compile = if out_bin.exists() {
                    let src_time = std::fs::metadata(native_src).and_then(|m| m.modified()).ok();
                    let bin_time = std::fs::metadata(&out_bin).and_then(|m| m.modified()).ok();
                    match (src_time, bin_time) {
                        (Some(st), Some(bt)) => st > bt,
                        _ => false,
                    }
                } else {
                    true
                };

                if should_compile {
                    let _ = std::process::Command::new("clang")
                        .args(["-O2", "-framework", "Cocoa", "-o", out_bin.to_str().unwrap(), native_src])
                        .status();
                }

                if !out_bin.exists() {
                    let _ = std::fs::write(&out_bin, b"");
                }
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        if let Ok(out_dir) = std::env::var("OUT_DIR") {
            let dummy = std::path::Path::new(&out_dir).join("devflow-dialog-macos");
            if !dummy.exists() {
                let _ = std::fs::write(&dummy, b"");
            }
        }
    }

    tauri_build::build();
}
