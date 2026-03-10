//! Build script that contains all the information to build the slint UI.

use std::{io, process::Command};

fn main() -> io::Result<()> {
    set_build_version();

    // Slint build
    let config = slint_build::CompilerConfiguration::new();
    slint_build::compile_with_config("ui/main.slint", config).expect("Slint build failed");

    Ok(())
}

fn set_build_version() {
    let build_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let jj_main_hash = Command::new("jj")
        .args([
            "log",
            "--no-graph",
            "-r",
            "main",
            "-T",
            "self.commit_id().short()"
        ])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or("unknown".to_string());

    let build_info = format!("build time: {}, jj main: {}", build_time, jj_main_hash);
    println!("cargo:rustc-env=BUILD_INFO={build_info}");
}
