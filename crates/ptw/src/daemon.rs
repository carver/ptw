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
use ptw_core::hotkey::{HotkeyEvent, HotkeyMachine, KeyEvent};
use ptw_core::session::{self, Input};
use ptw_core::typist::Typist;
use tracing::{error, info, warn};

use crate::audio::{Audio, Cue};
use crate::{dbus, engines, hotkey_source, portal_typist, tray};

/// Anything that can change what the daemon is doing.
#[derive(Debug)]
pub enum Command {
    Key(KeyEvent),
    Toggle,
    OpenSettings,
    Quit,
}

const CONFIG_POLL: Duration = Duration::from_secs(2);

struct Hold {
    input: Sender<Input>,
    worker: JoinHandle<()>,
}

struct Daemon {
    config_path: PathBuf,
    config: Config,
    config_mtime: Option<SystemTime>,
    hotkey: HotkeyMachine,
    words: CustomWords,
    engine: Arc<dyn Engine>,
    typist: Arc<Mutex<Box<dyn Typist>>>,
    audio: Audio,
    hold: Option<Hold>,
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
        TypistBackend::Portal => Box::new(runtime.block_on(
            portal_typist::PortalTypist::connect(runtime.handle().clone()),
        )?),
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
    let key_tx = commands.clone();
    let (keys_tx, keys_rx) = mpsc::channel::<KeyEvent>();
    hotkey_source::spawn(keys_tx);
    thread::spawn(move || {
        for key in keys_rx {
            if key_tx.send(Command::Key(key)).is_err() {
                break;
            }
        }
    });

    let mut daemon = Daemon {
        config_path: config_path.to_path_buf(),
        config_mtime: mtime(config_path),
        hotkey: HotkeyMachine::new(config.chord()?),
        words: CustomWords::new(&config.custom_words),
        config,
        engine,
        typist: Arc::new(Mutex::new(typist)),
        audio: Audio::spawn(),
        hold: None,
        tray,
        runtime: runtime.handle().clone(),
    };
    info!(hotkey = %daemon.hotkey.chord(), "ptw ready");
    daemon.serve(&inbox);
    Ok(())
}

fn mtime(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

impl Daemon {
    fn serve(&mut self, inbox: &Receiver<Command>) {
        loop {
            match inbox.recv_timeout(CONFIG_POLL) {
                Ok(Command::Key(key)) => match self.hotkey.on_key(key) {
                    Some(HotkeyEvent::Start) => self.start(),
                    Some(HotkeyEvent::Stop) => self.stop(),
                    Some(HotkeyEvent::Abort) => self.abort(),
                    None => {}
                },
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
            self.reload_config_if_changed();
        }
    }

    fn start(&mut self) {
        if self.hold.is_some() {
            return;
        }
        let (input, rx) = mpsc::channel::<Input>();
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
        if hold.worker.join().is_err() {
            error!("dictation thread panicked");
        }
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
                match config.chord() {
                    Ok(chord) => self.hotkey = HotkeyMachine::new(chord),
                    Err(e) => warn!(error = %e, "keeping the old hotkey"),
                }
                self.words = CustomWords::new(&config.custom_words);
                info!(hotkey = %self.hotkey.chord(), words = config.custom_words.len(), "config reloaded");
                self.config = config;
            }
            Err(e) => warn!(error = %e, "config not reloaded"),
        }
    }
}
