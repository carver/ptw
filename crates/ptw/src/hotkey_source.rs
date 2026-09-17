//! Reads keyboards under /dev/input, grabs them, and forwards what the
//! apps may see through a virtual keyboard (ADR 0006).
//!
//! One reader thread per keyboard runs the [`KeyProxy`] and writes to
//! uinput itself, so the daemon loop is never on the key path: a stalled
//! daemon cannot stall typing. A crashed reader drops its device, and the
//! kernel releases the grab with it. Without `/dev/uinput` access the
//! keyboards are read ungrabbed and the Hotkey leaks to the apps.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use evdev::uinput::VirtualDevice;
use evdev::{AttributeSet, Device, EventSummary, EventType, InputEvent, KeyCode};
use ptw_core::grab::{Grabbable, grab_when_idle};
use ptw_core::hotkey::{Chord, KeyAction, KeyEvent};
use ptw_core::proxy::{KeyProxy, Mode, Output};
use tracing::{debug, info, warn};

use crate::daemon::Command;

/// Name of the virtual keyboard the uinput Typist creates.
pub const OWN_DEVICE_NAME: &str = "ptw virtual keyboard";
/// Name of the virtual keyboard the Key proxy forwards through.
pub const PROXY_DEVICE_NAME: &str = "ptw keyboard proxy";

const RESCAN_EVERY: Duration = Duration::from_secs(3);
/// The grab waits for every key to come up, however long that takes
/// (`ptw_core::grab`); past this it says so in the log.
const KEYS_UP_WARN_AFTER: Duration = Duration::from_secs(1);
const KEYS_UP_POLL: Duration = Duration::from_millis(10);

/// Our own virtual keyboards must never be read back as Hotkey input.
pub fn is_keyboard(device: &Device) -> bool {
    let is_ours = device.name().is_some_and(|n| n.starts_with("ptw "));
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

pub fn uinput_writable() -> bool {
    std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/uinput")
        .is_ok()
}

struct State {
    proxy: KeyProxy,
    virtual_keyboard: Option<VirtualDevice>,
    commands: Sender<Command>,
}

impl State {
    fn dispatch(&mut self, outputs: Vec<Output>) {
        for output in outputs {
            match output {
                Output::Forward(event) => {
                    if let Some(keyboard) = &mut self.virtual_keyboard
                        && let Err(e) = keyboard.emit(&[to_input_event(event)])
                    {
                        warn!(error = %e, ?event, "cannot forward key");
                    }
                }
                Output::Hotkey(event) => {
                    self.commands.send(Command::Hotkey(event)).ok();
                }
                Output::ModifiersHeld(held) => {
                    self.commands.send(Command::ModifiersHeld(held)).ok();
                }
            }
        }
    }
}

struct Shared {
    state: Mutex<State>,
    /// Signalled whenever the proxy may have a new deadline.
    wake: Condvar,
}

impl Shared {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// Handle to the reader threads.
pub struct HotkeySource {
    shared: Arc<Shared>,
}

impl HotkeySource {
    /// Starts reading. Returns once the first scan is done.
    pub fn spawn(chord: Chord, deferral: Duration, commands: Sender<Command>) -> Self {
        let (virtual_keyboard, mode) = match open_virtual_keyboard() {
            Ok(keyboard) => (Some(keyboard), Mode::Grab),
            Err(e) => {
                warn!(
                    error = %e,
                    "no /dev/uinput access: the Hotkey will leak to apps and text waits for modifiers; run `ptw setup`"
                );
                (None, Mode::PassThrough)
            }
        };
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                proxy: KeyProxy::new(chord, mode, deferral),
                virtual_keyboard,
                commands,
            }),
            wake: Condvar::new(),
        });
        let open: Arc<Mutex<HashSet<PathBuf>>> = Arc::default();
        scan(&shared, &open);
        if open.lock().unwrap().is_empty() {
            warn!(
                "no readable keyboard under /dev/input: the Hotkey cannot work. Not in the `input` group? A systemd user service only has the groups you had at login, so joining a group needs a logout"
            );
        }
        let rescan = Arc::clone(&shared);
        thread::Builder::new()
            .name("ptw-hotkey-rescan".into())
            .spawn(move || {
                loop {
                    thread::sleep(RESCAN_EVERY);
                    scan(&rescan, &open);
                }
            })
            .expect("spawn rescan thread");
        let timer = Arc::clone(&shared);
        thread::Builder::new()
            .name("ptw-hotkey-deferral".into())
            .spawn(move || run_deadlines(&timer))
            .expect("spawn deferral thread");
        Self { shared }
    }

    pub fn mode(&self) -> Mode {
        self.shared.lock().proxy.mode()
    }

    pub fn chord(&self) -> Chord {
        self.shared.lock().proxy.chord().clone()
    }

    pub fn set_chord(&self, chord: Chord) {
        let mut state = self.shared.lock();
        let outputs = state.proxy.set_chord(chord);
        state.dispatch(outputs);
    }

    pub fn set_deferral(&self, deferral: Duration) {
        self.shared.lock().proxy.set_deferral(deferral);
        self.shared.wake.notify_one();
    }
}

fn open_virtual_keyboard() -> anyhow::Result<VirtualDevice> {
    let mut keys = AttributeSet::<KeyCode>::new();
    // Every KEY_* code, skipping the BTN_* range so libinput does not take
    // the device for a mouse or joystick.
    for code in (1..=0xff).chain(0x160..=0x2ff) {
        keys.insert(KeyCode(code));
    }
    Ok(VirtualDevice::builder()?
        .name(PROXY_DEVICE_NAME)
        .with_keys(&keys)?
        .build()?)
}

fn to_input_event(event: KeyEvent) -> InputEvent {
    let value = match event.action {
        KeyAction::Release => 0,
        KeyAction::Press => 1,
        KeyAction::Repeat => 2,
    };
    InputEvent::new(EventType::KEY.0, event.code.0, value)
}

fn scan(shared: &Arc<Shared>, open: &Arc<Mutex<HashSet<PathBuf>>>) {
    for (path, device) in evdev::enumerate() {
        if !is_keyboard(&device) || !open.lock().unwrap().insert(path.clone()) {
            continue;
        }
        info!(path = %path.display(), name = device.name().unwrap_or("?"), "reading keyboard");
        let shared = Arc::clone(shared);
        let open = Arc::clone(open);
        thread::Builder::new()
            .name(format!("ptw-hotkey-{}", path.display()))
            .spawn(move || {
                read_until_error(device, &shared);
                open.lock().unwrap().remove(&path);
            })
            .expect("spawn keyboard thread");
    }
}

fn read_until_error(mut device: Device, shared: &Shared) {
    let name = device.name().unwrap_or("?").to_string();
    if shared.lock().proxy.mode() == Mode::Grab {
        match grab_once_keys_are_up(&mut device, &name) {
            Ok(()) => info!(name, "keyboard grabbed"),
            Err(e) => warn!(name, error = %e, "cannot grab keyboard; its Hotkey will leak"),
        }
    }
    loop {
        let events = match device.fetch_events() {
            Ok(events) => events,
            Err(e) => {
                warn!(name, error = %e, "keyboard went away");
                return;
            }
        };
        for event in events {
            let EventSummary::Key(_, key, value) = event.destructure() else {
                continue;
            };
            let action = match value {
                0 => KeyAction::Release,
                1 => KeyAction::Press,
                _ => KeyAction::Repeat,
            };
            let event = KeyEvent {
                code: ptw_core::keys::KeyCode(key.0),
                action,
            };
            let mut state = shared.lock();
            let outputs = state.proxy.on_key(event, Instant::now());
            state.dispatch(outputs);
            let has_deadline = state.proxy.deadline().is_some();
            drop(state);
            if has_deadline {
                shared.wake.notify_one();
            }
        }
    }
}

/// A grab taken while a key is down leaves that key dead for the whole
/// desktop (`ptw_core::grab`), so this waits, however long it takes.
fn grab_once_keys_are_up(device: &mut Device, name: &str) -> std::io::Result<()> {
    let started = Instant::now();
    let mut warned = false;
    grab_when_idle(&mut Physical(device), |down| {
        let keys = down
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ");
        if !warned && started.elapsed() >= KEYS_UP_WARN_AFTER {
            warned = true;
            warn!(
                name,
                keys, "keys held down; the grab waits for them to come up"
            );
        } else {
            debug!(keys, "waiting for keys to come up before grabbing");
        }
        thread::sleep(KEYS_UP_POLL);
    })?;
    if warned {
        info!(
            name,
            waited_ms = started.elapsed().as_millis(),
            "keys came up"
        );
    }
    Ok(())
}

/// A real keyboard under /dev/input, as `ptw_core::grab` sees it.
struct Physical<'a>(&'a mut Device);

impl Grabbable for Physical<'_> {
    type Error = std::io::Error;

    fn keys_down(&mut self) -> std::io::Result<Vec<ptw_core::keys::KeyCode>> {
        Ok(self
            .0
            .get_key_state()?
            .iter()
            .map(|key| ptw_core::keys::KeyCode(key.0))
            .collect())
    }

    fn grab(&mut self) -> std::io::Result<()> {
        self.0.grab()
    }

    fn ungrab(&mut self) -> std::io::Result<()> {
        self.0.ungrab()
    }
}

/// Forwards deferred modifier presses whose deadline has passed.
fn run_deadlines(shared: &Shared) {
    let mut state = shared.lock();
    loop {
        match state.proxy.deadline() {
            None => {
                state = shared.wake.wait(state).unwrap_or_else(|e| e.into_inner());
            }
            Some(deadline) => {
                let now = Instant::now();
                if now >= deadline {
                    let outputs = state.proxy.on_deadline(now);
                    state.dispatch(outputs);
                } else {
                    state = shared
                        .wake
                        .wait_timeout(state, deadline - now)
                        .unwrap_or_else(|e| e.into_inner())
                        .0;
                }
            }
        }
    }
}
