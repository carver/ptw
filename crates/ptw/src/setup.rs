//! `ptw setup`: the one-time host steps, and launching the settings dialog.

use std::path::Path;
use std::process::Command;

use anyhow::Context;
use clap::Args as ClapArgs;
use ptw_core::config::Config;

#[derive(ClapArgs)]
pub struct Args {
    /// Only print what would be done.
    #[arg(long)]
    pub dry_run: bool,
}

const UNIT: &str = "\
[Unit]
Description=Push to Whisper
After=graphical-session.target
PartOf=graphical-session.target

[Service]
ExecStart=%h/.cargo/bin/ptw daemon
Restart=on-failure
RestartSec=2

[Install]
WantedBy=graphical-session.target
";

pub fn run(config_path: &Path, args: Args) -> anyhow::Result<()> {
    let config = Config::load(config_path)?;
    if !config_path.exists() {
        println!("writing default config to {}", config_path.display());
        if !args.dry_run {
            config.save(config_path)?;
        }
    }

    let in_input_group = std::process::Command::new("id")
        .arg("-nG")
        .output()
        .is_ok_and(|o| {
            String::from_utf8_lossy(&o.stdout)
                .split_whitespace()
                .any(|g| g == "input")
        });
    if in_input_group {
        println!("ok   you are in the `input` group");
    } else {
        println!(
            "TODO the hotkey needs read access to /dev/input. Run:\n       sudo usermod -aG input $USER\n     then log out and back in. Note: this lets any process running as you read every keystroke."
        );
    }

    let unit_path = ptw_core::config::config_dir()
        .parent()
        .unwrap()
        .join("systemd/user/ptw.service");
    println!("writing {}", unit_path.display());
    if !args.dry_run {
        std::fs::create_dir_all(unit_path.parent().unwrap())?;
        std::fs::write(&unit_path, UNIT)?;
        let status = Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .status()
            .context("systemctl --user")?;
        if !status.success() {
            println!("TODO `systemctl --user daemon-reload` failed; enable the unit by hand");
        }
        println!("enable and start it with: systemctl --user enable --now ptw");
    }
    println!(
        "models: `ptw setup` downloads nothing yet; the engine step lands with the engine choice"
    );
    Ok(())
}

/// Launches `ptw settings` as its own process so a GUI crash cannot take
/// the daemon down.
pub fn open_settings(config_path: &Path) -> anyhow::Result<()> {
    let exe = std::env::current_exe()?;
    Command::new(exe)
        .arg("--config")
        .arg(config_path)
        .arg("settings")
        .spawn()
        .context("spawn settings")?;
    Ok(())
}
