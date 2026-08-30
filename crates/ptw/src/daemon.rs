//! The daemon: one control loop that owns the Dictation state, fed by the
//! hotkey reader, D-Bus, the tray, and a config-file poll.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime};

use anyhow::Context;
use ptw_core::config::{Config, TypistBackend};
use ptw_core::correction::CustomWords;
use ptw_core::engine::Engine;
use ptw_core::hotkey::HotkeyEvent;
use ptw_core::layout::Layout;
use ptw_core::session::{self, Input};
use ptw_core::typist::Typist;
use tracing::{error, info, warn};

use crate::audio::{Audio, Cue};
use crate::hotkey_source::HotkeySource;
use crate::{dbus, engines, layout, portal_typist, tray};

/// Anything that can change what the daemon is doing.
#[derive(Debug)]
pub enum Command {
    Hotkey(HotkeyEvent),
    /// The compositor now believes a modifier key is down, or none are.
    ModifiersHeld(bool),
    Toggle,
    OpenSettings,
    Quit,
}

const CONFIG_POLL: Duration = Duration::from_secs(2);

struct Hold {
    input: Sender<Input>,
    worker: JoinHandle<()>,
}

impl Hold {
    fn report(self) {
        if self.worker.join().is_err() {
            error!("dictation thread panicked");
        }
    }
}

struct Daemon {
    config_path: PathBuf,
    config: Config,
    config_mtime: Option<SystemTime>,
    layout: Layout,
    source: HotkeySource,
    modifiers_held: bool,
    words: CustomWords,
    engine: Arc<dyn Engine>,
    typist: Arc<Mutex<Box<dyn Typist>>>,
    audio: Audio,
    hold: Option<Hold>,
    /// Holds that have stopped but may still be waiting to type: the
    /// chord's modifier is usually still down when the Hold ends.
    finishing: Vec<Hold>,
    tray: Option<ksni::Handle<tray::PtwTray>>,
    runtime: tokio::runtime::Handle,
}

pub fn run(config_path: &Path) -> anyhow::Result<()> {
    let config = Config::load(config_path)?;
    let runtime = tokio::runtime::Runtime::new()?;
    let (commands, inbox) = mpsc::channel::<Command>();

    let _bus = runtime.block_on(dbus::serve(commands.clone()))?;
    let engine = engines::build(&config.engine)?;
    info!(engine = engine.name(), "engine loaded");
    let typist: Box<dyn Typist> = match config.typist.backend {
        TypistBackend::Portal => Box::new(
            runtime
                .block_on(portal_typist::PortalTypist::connect(
                    runtime.handle().clone(),
                ))
                .with_context(|| match crate::doctor::foreign_primary_group() {
                    Some(group) => format!(
                        "open the portal keyboard session (this shell's primary group is `{group}`, so it came from `newgrp`/`sg`; the portal refuses such callers. Log out and back in instead)"
                    ),
                    None => "open the portal keyboard session".to_string(),
                })?,
        ),
        TypistBackend::Uinput => Box::new(
            crate::uinput_typist::UinputTypist::open()
                .context("open /dev/uinput; run `ptw setup`")?,
        ),
    };
    let tray = match runtime.block_on(tray::spawn(commands.clone())) {
        Ok(handle) => Some(handle),
        Err(e) => {
            warn!(error = %e, "no tray (is the AppIndicator extension enabled?); continuing without");
            None
        }
    };
    let layout = layout::detect();
    let source = HotkeySource::spawn(config.chord_in(&layout)?, commands.clone());
    let mut daemon = Daemon {
        config_path: config_path.to_path_buf(),
        config_mtime: mtime(config_path),
        source,
        modifiers_held: false,
        layout,
        words: CustomWords::new(&config.custom_words),
        config,
        engine,
        typist: Arc::new(Mutex::new(typist)),
        audio: Audio::spawn(),
        hold: None,
        finishing: Vec::new(),
        tray,
        runtime: runtime.handle().clone(),
    };
    daemon.log_hotkey("ptw ready");
    daemon.serve(&inbox);
    Ok(())
}

fn mtime(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

impl Daemon {
    fn log_hotkey(&self, what: &str) {
        let chord = self.source.chord();
        info!(
            hotkey = %chord,
            physical = chord.physical(),
            layout = self.layout.name(),
            mode = ?self.source.mode(),
            what
        );
    }

    fn serve(&mut self, inbox: &Receiver<Command>) {
        loop {
            match inbox.recv_timeout(CONFIG_POLL) {
                Ok(Command::Hotkey(HotkeyEvent::Start)) => self.start(),
                Ok(Command::Hotkey(HotkeyEvent::Stop)) => self.stop(),
                Ok(Command::Hotkey(HotkeyEvent::Abort)) => self.abort(),
                Ok(Command::ModifiersHeld(held)) => {
                    self.modifiers_held = held;
                    for hold in self.hold.iter().chain(&self.finishing) {
                        hold.input.send(Input::ModifiersHeld(held)).ok();
                    }
                }
                Ok(Command::Toggle) => {
                    if self.hold.is_some() {
                        self.stop();
                    } else {
                        self.start();
                    }
                }
                Ok(Command::OpenSettings) => {
                    if let Err(e) = crate::setup::open_settings(&self.config_path) {
                        warn!(error = %e, "cannot open settings");
                    }
                }
                Ok(Command::Quit) | Err(RecvTimeoutError::Disconnected) => {
                    self.abort();
                    info!("quitting");
                    return;
                }
                Err(RecvTimeoutError::Timeout) => {}
            }
            self.reap_finished();
            self.reload_config_if_changed();
        }
    }

    fn reap_finished(&mut self) {
        let (done, waiting): (Vec<Hold>, Vec<Hold>) = self
            .finishing
            .drain(..)
            .partition(|hold| hold.worker.is_finished());
        self.finishing = waiting;
        done.into_iter().for_each(Hold::report);
    }

    fn start(&mut self) {
        if self.hold.is_some() {
            return;
        }
        let (input, rx) = mpsc::channel::<Input>();
        input.send(Input::ModifiersHeld(self.modifiers_held)).ok();
        let engine = Arc::clone(&self.engine);
        let words = self.words.clone();
        let typist = Arc::clone(&self.typist);
        let worker = thread::Builder::new()
            .name("ptw-dictation".into())
            .spawn(move || {
                let mut typist = typist.lock().unwrap();
                match session::run(&*engine, &words, &rx, &mut **typist) {
                    Ok(outcome) => info!(
                        typed = outcome.typed,
                        dropped = outcome.dropped,
                        aborted = outcome.aborted,
                        "dictation done"
                    ),
                    Err(e) => error!(error = %e, "dictation failed"),
                }
            })
            .expect("spawn dictation thread");
        self.audio
            .start_capture(&self.config.audio.device, input.clone());
        if self.config.audio.cues {
            self.audio.play(Cue::Start);
        }
        self.hold = Some(Hold { input, worker });
        self.set_tray_active(true);
        info!("hold started");
    }

    fn stop(&mut self) {
        self.end_hold(Input::Stop, Cue::Stop);
    }

    fn abort(&mut self) {
        self.end_hold(Input::Abort, Cue::Abort);
    }

    fn end_hold(&mut self, message: Input, cue: Cue) {
        let Some(hold) = self.hold.take() else { return };
        self.audio.stop_capture();
        hold.input.send(message).ok();
        if self.config.audio.cues {
            self.audio.play(cue);
        }
        self.finishing.push(hold);
        self.set_tray_active(false);
    }

    fn set_tray_active(&self, active: bool) {
        if let Some(handle) = &self.tray {
            self.runtime.block_on(tray::set_active(handle, active));
        }
    }

    fn reload_config_if_changed(&mut self) {
        let now = mtime(&self.config_path);
        if now == self.config_mtime {
            return;
        }
        self.config_mtime = now;
        match Config::load(&self.config_path) {
            Ok(config) => {
                if config.engine != self.config.engine || config.typist != self.config.typist {
                    warn!("engine or typist settings changed; restart the daemon to apply them");
                }
                self.layout = layout::detect();
                match config.chord_in(&self.layout) {
                    Ok(chord) => self.source.set_chord(chord),
                    Err(e) => warn!(error = %e, "keeping the old hotkey"),
                }
                self.words = CustomWords::new(&config.custom_words);
                self.config = config;
                self.log_hotkey("config reloaded");
            }
            Err(e) => warn!(error = %e, "config not reloaded"),
        }
    }
}
