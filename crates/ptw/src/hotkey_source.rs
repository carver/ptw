//! Reads keyboards under /dev/input and forwards presses and releases.
//!
//! One thread per keyboard, plus a slow rescan so a keyboard plugged in
//! later is picked up. Needs the `input` group (ADR 0003).

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use evdev::{Device, EventSummary, KeyCode};
use ptw_core::hotkey::{KeyAction, KeyEvent};
use tracing::{debug, info, warn};

/// Name of the virtual keyboard the uinput Typist creates, so its own
/// keystrokes are not read back as Hotkey input.
pub const OWN_DEVICE_NAME: &str = "ptw virtual keyboard";

const RESCAN_EVERY: Duration = Duration::from_secs(3);

pub fn is_keyboard(device: &Device) -> bool {
    let is_ours = device.name().is_some_and(|n| n == OWN_DEVICE_NAME);
    !is_ours
        && device
            .supported_keys()
            .is_some_and(|keys| keys.contains(KeyCode::KEY_A) && keys.contains(KeyCode::KEY_Z))
}

/// Keyboards visible right now, with whether each one can be read.
pub fn list_keyboards() -> Vec<(PathBuf, String, bool)> {
    evdev::enumerate()
        .filter(|(_, d)| is_keyboard(d))
        .map(|(path, d)| {
            let readable = std::fs::File::open(&path).is_ok();
            (path, d.name().unwrap_or("?").to_string(), readable)
        })
        .collect()
}

/// Starts the reader threads. Returns once the first scan is done.
pub fn spawn(tx: Sender<KeyEvent>) {
    let open: Arc<Mutex<HashSet<PathBuf>>> = Arc::default();
    scan(&tx, &open);
    thread::Builder::new()
        .name("ptw-hotkey-rescan".into())
        .spawn(move || {
            loop {
                thread::sleep(RESCAN_EVERY);
                scan(&tx, &open);
            }
        })
        .expect("spawn rescan thread");
}

fn scan(tx: &Sender<KeyEvent>, open: &Arc<Mutex<HashSet<PathBuf>>>) {
    for (path, device) in evdev::enumerate() {
        if !is_keyboard(&device) || !open.lock().unwrap().insert(path.clone()) {
            continue;
        }
        info!(path = %path.display(), name = device.name().unwrap_or("?"), "reading keyboard");
        let tx = tx.clone();
        let open = Arc::clone(open);
        thread::Builder::new()
            .name(format!("ptw-hotkey-{}", path.display()))
            .spawn(move || {
                read_until_error(device, &tx);
                open.lock().unwrap().remove(&path);
            })
            .expect("spawn keyboard thread");
    }
}

fn read_until_error(mut device: Device, tx: &Sender<KeyEvent>) {
    let name = device.name().unwrap_or("?").to_string();
    loop {
        let events = match device.fetch_events() {
            Ok(events) => events,
            Err(e) => {
                warn!(name, error = %e, "keyboard went away");
                return;
            }
        };
        for event in events {
            if let EventSummary::Key(_, key, value) = event.destructure() {
                let action = match value {
                    0 => KeyAction::Release,
                    1 => KeyAction::Press,
                    _ => KeyAction::Repeat,
                };
                let event = KeyEvent {
                    code: ptw_core::keys::KeyCode(key.0),
                    action,
                };
                if tx.send(event).is_err() {
                    debug!("hotkey receiver gone; stopping reader");
                    return;
                }
            }
        }
    }
}
