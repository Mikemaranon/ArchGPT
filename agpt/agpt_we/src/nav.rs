// nav.rs
// Module for detecting OS, checking Chromium installation, and installing it if missing

use std::process::Command;
use std::fs;

/// Check if Chromium is already installed on the system
pub fn chromium_exists() -> bool {
    let candidates = ["chromium", "chromium-browser", "google-chrome"];
    candidates.iter().any(|cmd| Command::new(cmd).arg("--version").output().is_ok())
}

/// Detect Linux distribution by reading /etc/os-release
fn detect_linux_distro() -> Option<String> {
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("ID=") {
                // Remove quotes if present
                return Some(line[3..].trim_matches('"').to_string());
            }
        }
    }
    None
}

/// Install Chromium using the appropriate package manager for the detected OS/distro
pub fn install_chromium() {
    if cfg!(target_os = "linux") {
        if let Some(distro) = detect_linux_distro() {
            let mut cmd = Command::new("sudo");

            match distro.as_str() {
                "arch" | "manjaro" => {
                    cmd.arg("pacman").arg("-S").arg("--noconfirm").arg("chromium");
                },
                "ubuntu" | "debian" => {
                    cmd.arg("apt-get").arg("install").arg("-y").arg("chromium-browser");
                },
                "fedora" => {
                    cmd.arg("dnf").arg("install").arg("-y").arg("chromium");
                },
                other => {
                    eprintln!("Unsupported Linux distro: {}", other);
                    return;
                }
            }

            let status = cmd.status().expect("Failed to run installer");
            if status.success() {
                println!("✅ Chromium installed successfully!");
            } else {
                eprintln!("❌ Failed to install Chromium");
            }
        } else {
            eprintln!("❌ Could not detect Linux distro");
        }
    } else if cfg!(target_os = "windows") {
        eprintln!("Windows: please install Chromium manually or via winget/choco");
    } else if cfg!(target_os = "macos") {
        eprintln!("MacOS: please install Chromium manually or via brew");
    }
}

/// Main helper function to ensure Chromium is installed
pub fn ensure_chromium() {
    if !chromium_exists() {
        println!("Chromium not found. Installing...");
        install_chromium();
    } else {
        println!("Chromium already installed");
    }
}
