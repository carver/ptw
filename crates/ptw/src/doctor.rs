//! `ptw doctor`: what works on this machine, one line per check.

use std::path::Path;
use std::process::Command;

use ptw_core::config::Config;

use crate::{audio, dbus, engines, hotkey_source, layout, portal_typist};

fn line(ok: bool, what: &str, detail: impl AsRef<str>) {
    println!(
        "{} {:<12} {}",
        if ok { "ok  " } else { "FAIL" },
        what,
        detail.as_ref()
    );
}

pub fn run(config_path: &Path) -> anyhow::Result<()> {
    match Config::load(config_path) {
        Ok(config) => {
            line(
                true,
                "config",
                format!(
                    "{} (hotkey {}, {} custom words)",
                    config_path.display(),
                    config.hotkey,
                    config.custom_words.len()
                ),
            );
            let layout = layout::detect();
            match config.chord_in(&layout) {
                Ok(chord) => line(
                    true,
                    "layout",
                    format!(
                        "{} (hotkey {} is physical {})",
                        layout.name(),
                        chord,
                        chord.physical()
                    ),
                ),
                Err(e) => line(false, "layout", format!("{} ({e})", layout.name())),
            }
            let engine_ok = engines::build(&config.engine).is_ok();
            line(
                engine_ok,
                "engine",
                format!(
                    "{} / {} (available: {})",
                    config.engine.kind,
                    config.engine.model,
                    engines::available().join(", ")
                ),
            );
        }
        Err(e) => line(false, "config", e.to_string()),
    }

    let keyboards = hotkey_source::list_keyboards();
    if keyboards.is_empty() {
        line(
            false,
            "keyboards",
            "none found under /dev/input (not in the `input` group? run `ptw setup`)",
        );
    }
    for (path, name, readable) in keyboards {
        line(
            readable,
            "keyboard",
            format!(
                "{name} at {}{}",
                path.display(),
                if readable {
                    ""
                } else {
                    " (not readable: join the `input` group)"
                }
            ),
        );
    }

    if let Some(groups) = service_groups() {
        let has_input = groups.iter().any(|g| g == "input");
        line(
            has_input,
            "service",
            if has_input {
                "systemd user services have the `input` group".to_string()
            } else {
                format!(
                    "systemd user services lack the `input` group (they have: {}); log out and back in so `systemctl --user start ptw` can read the keyboard",
                    groups.join(" ")
                )
            },
        );
    }

    let inputs = audio::list_inputs();
    line(
        !inputs.is_empty(),
        "microphones",
        if inputs.is_empty() {
            "none".to_string()
        } else {
            inputs.join(", ")
        },
    );

    match foreign_primary_group() {
        None => line(true, "group", "primary group is your login group"),
        Some(group) => line(
            false,
            "group",
            format!(
                "primary group is `{group}`, not your login group: this shell came from `newgrp`/`sg`, and the portal refuses such callers. Log out and back in instead."
            ),
        ),
    }

    let runtime = tokio::runtime::Runtime::new()?;
    match runtime.block_on(portal_typist::probe()) {
        Ok(detail) => line(true, "portal", detail),
        Err(e) => line(false, "portal", e.to_string()),
    }
    line(
        hotkey_source::uinput_writable(),
        "uinput",
        if hotkey_source::uinput_writable() {
            "/dev/uinput writable: the Hotkey is grabbed and streams while held"
        } else if Path::new("/dev/uinput").exists() {
            "/dev/uinput not writable: the Hotkey leaks to apps and text waits for its release; run `ptw setup`"
        } else {
            "/dev/uinput missing: run `ptw setup`"
        },
    );

    let wayland = std::env::var_os("WAYLAND_DISPLAY").is_some();
    line(
        true,
        "session",
        if wayland { "Wayland" } else { "X11 or unknown" },
    );
    line(
        true,
        "daemon",
        if dbus::daemon_running() {
            "running"
        } else {
            "not running"
        },
    );
    Ok(())
}

/// The name of the process's primary group when it differs from the login
/// group in passwd, as after `newgrp input` or `sg input`.
///
/// xdg-desktop-portal identifies a caller by opening `/proc/<pid>/root`,
/// which the kernel allows only when uid and gid both match the portal's
/// own. A `newgrp` shell fails that check with "Unable to open /proc/<pid>/root".
pub fn foreign_primary_group() -> Option<String> {
    let id = |flag: &str| {
        Command::new("id")
            .arg(flag)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    };
    let uid = id("-u")?;
    let gid = id("-g")?;
    let passwd = Command::new("getent")
        .args(["passwd", &uid])
        .output()
        .ok()?;
    let passwd = String::from_utf8_lossy(&passwd.stdout);
    let login_gid = passwd.trim().split(':').nth(3)?;
    (gid != login_gid).then(|| id("-gn").unwrap_or(gid))
}

/// The groups a systemd user service runs with: those of the user manager,
/// fixed at login, not those of the calling shell.
fn service_groups() -> Option<Vec<String>> {
    let out = Command::new("systemd-run")
        .args([
            "--user",
            "--quiet",
            "--wait",
            "--pipe",
            "--collect",
            "id",
            "-nG",
        ])
        .output()
        .ok()
        .filter(|o| o.status.success())?;
    Some(
        String::from_utf8_lossy(&out.stdout)
            .split_whitespace()
            .map(String::from)
            .collect(),
    )
}
