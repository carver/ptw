//! Finds the user's keyboard layout and turns it into a [`Layout`].
//!
//! GNOME keeps the layout in gsettings (`org.gnome.desktop.input-sources`);
//! elsewhere `localectl` knows the X11 layout. libxkbcommon then tells us
//! which key types which character.

use std::process::Command;

use ptw_core::keys::KeyCode;
use ptw_core::layout::Layout;
use tracing::{debug, warn};
use xkbcommon::xkb;

/// evdev key codes are XKB keycodes minus 8.
const EVDEV_OFFSET: u32 = 8;

pub fn detect() -> Layout {
    let Some((layout, variant)) = gnome_source().or_else(localectl) else {
        debug!("keyboard layout unknown; assuming US QWERTY");
        return Layout::qwerty();
    };
    compile(&layout, &variant).unwrap_or_else(|| {
        warn!(
            layout,
            variant, "cannot compile keyboard layout; assuming US QWERTY"
        );
        Layout::qwerty()
    })
}

/// The most recently used xkb input source, else the first configured one.
fn gnome_source() -> Option<(String, String)> {
    ["mru-sources", "sources"].iter().find_map(|key| {
        let out = Command::new("gsettings")
            .args(["get", "org.gnome.desktop.input-sources", key])
            .output()
            .ok()
            .filter(|o| o.status.success())?;
        first_xkb_source(&String::from_utf8_lossy(&out.stdout))
    })
}

/// `[('xkb', 'us+dvorak'), ('ibus', 'anthy')]` → `("us", "dvorak")`.
fn first_xkb_source(text: &str) -> Option<(String, String)> {
    let spec = text.split("('xkb', '").nth(1)?.split('\'').next()?;
    Some(split_variant(spec))
}

fn split_variant(spec: &str) -> (String, String) {
    match spec.split_once('+') {
        Some((layout, variant)) => (layout.to_string(), variant.to_string()),
        None => (spec.to_string(), String::new()),
    }
}

fn localectl() -> Option<(String, String)> {
    let out = Command::new("localectl").arg("status").output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let field = |name: &str| {
        text.lines()
            .find_map(|l| l.trim().strip_prefix(name))
            .and_then(|v| v.split(',').next())
            .map(|v| v.trim().to_string())
    };
    let layout = field("X11 Layout:")?;
    Some((layout, field("X11 Variant:").unwrap_or_default()))
}

fn compile(layout: &str, variant: &str) -> Option<Layout> {
    let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
    let keymap = xkb::Keymap::new_from_names(
        &context,
        "",
        "",
        layout,
        variant,
        None,
        xkb::KEYMAP_COMPILE_NO_FLAGS,
    )?;
    let name = if variant.is_empty() {
        layout.to_string()
    } else {
        format!("{layout}+{variant}")
    };
    let mut keys = Vec::new();
    for raw in keymap.min_keycode().raw()..=keymap.max_keycode().raw() {
        let Some(code) = raw
            .checked_sub(EVDEV_OFFSET)
            .and_then(|c| u16::try_from(c).ok())
        else {
            continue;
        };
        for sym in keymap.key_get_syms_by_level(xkb::Keycode::new(raw), 0, 0) {
            if let Some(ch) = sym.key_char() {
                keys.push((ch, KeyCode(code)));
            }
        }
    }
    Some(Layout::new(name, keys))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ptw_core::keys::key_code;

    #[test]
    fn parses_gnome_input_sources() {
        assert_eq!(
            first_xkb_source("[('xkb', 'us+dvorak')]"),
            Some(("us".into(), "dvorak".into()))
        );
        assert_eq!(
            first_xkb_source("[('ibus', 'anthy'), ('xkb', 'de')]"),
            Some(("de".into(), String::new()))
        );
        assert_eq!(first_xkb_source("@a(ss) []"), None);
    }

    #[test]
    fn dvorak_puts_z_under_qwertys_slash() {
        let dvorak = compile("us", "dvorak").expect("xkb data installed");
        assert_eq!(dvorak.name(), "us+dvorak");
        assert_eq!(dvorak.codes('z'), Some(&[key_code("Slash").unwrap()][..]));
        assert_eq!(dvorak.codes(';'), Some(&[key_code("z").unwrap()][..]));
        let qwerty = compile("us", "").unwrap();
        assert_eq!(qwerty.codes('z'), Some(&[key_code("z").unwrap()][..]));
    }
}
