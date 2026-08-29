//! `ptw`: the daemon and its command-line verbs.

mod audio;
mod daemon;
mod dbus;
mod doctor;
mod engines;
mod hotkey_source;
mod portal_typist;
mod settings_ui;
mod setup;
mod transcribe;
mod tray;
mod uinput_typist;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "ptw",
    version,
    about = "Push to Whisper: hold a key, talk, and the words appear where you are typing"
)]
struct Cli {
    /// Config file (default: ~/.config/ptw/config.toml).
    #[arg(long, global = true)]
    config: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the daemon: hotkey, audio, Engine, tray.
    Daemon,
    /// One-time host setup: model download, systemd user unit, permission checks.
    Setup(setup::Args),
    /// Report what works on this machine.
    Doctor,
    /// Open the settings dialog.
    Settings,
    /// Start a Dictation if idle, stop it if one is running (for a desktop shortcut).
    Toggle,
    /// Transcribe a WAV file the way the daemon would, printing text as it commits.
    Transcribe(transcribe::Args),
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with_writer(std::io::stderr)
        .init();
    let cli = Cli::parse();
    let config_path = cli.config.unwrap_or_else(ptw_core::config::config_path);
    match cli.command {
        Command::Daemon => daemon::run(&config_path),
        Command::Setup(args) => setup::run(&config_path, args),
        Command::Doctor => doctor::run(&config_path),
        Command::Settings => settings_ui::run(&config_path),
        Command::Toggle => dbus::toggle(),
        Command::Transcribe(args) => transcribe::run(&config_path, args),
    }
}
