//! `ptw doctor`: what works on this machine, one line per check.

use std::path::Path;

use ptw_core::config::Config;

use crate::{audio, dbus, engines, hotkey_source, portal_typist};

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

    let runtime = tokio::runtime::Runtime::new()?;
    match runtime.block_on(portal_typist::probe()) {
        Ok(detail) => line(true, "portal", detail),
        Err(e) => line(false, "portal", e.to_string()),
    }
    line(
        Path::new("/dev/uinput").exists(),
        "uinput",
        "/dev/uinput (only needed for the uinput typist)",
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
