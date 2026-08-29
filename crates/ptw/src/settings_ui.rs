//! `ptw settings`: a small form over the config file. Saving writes the
//! file; the daemon notices within a couple of seconds.

use std::path::{Path, PathBuf};
use std::process::Command;

use eframe::egui;
use ptw_core::config::{Config, TypistBackend};
use ptw_core::hotkey::Chord;

use crate::{audio, engines};

const LOOKAHEADS_MS: [u32; 4] = [80, 160, 560, 1120];

struct App {
    path: PathBuf,
    config: Config,
    saved: Config,
    custom_words_text: String,
    microphones: Vec<String>,
    status: String,
}

impl App {
    fn new(path: PathBuf) -> anyhow::Result<Self> {
        let config = Config::load(&path)?;
        Ok(Self {
            path,
            custom_words_text: config.custom_words.join("\n"),
            saved: config.clone(),
            config,
            microphones: audio::list_inputs(),
            status: String::new(),
        })
    }

    fn sync_custom_words(&mut self) {
        self.config.custom_words = self
            .custom_words_text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(String::from)
            .collect();
    }

    fn hotkey_problem(&self) -> Option<String> {
        self.config
            .hotkey
            .parse::<Chord>()
            .err()
            .map(|e| e.to_string())
    }

    fn save(&mut self) {
        self.sync_custom_words();
        if let Some(problem) = self.hotkey_problem() {
            self.status = format!("not saved: {problem}");
            return;
        }
        match self.config.save(&self.path) {
            Ok(()) => {
                let needs_restart = self.config.engine != self.saved.engine
                    || self.config.typist != self.saved.typist;
                self.saved = self.config.clone();
                self.status = if needs_restart {
                    "saved; engine or typist changes need a daemon restart".into()
                } else {
                    "saved; the daemon picks it up within a few seconds".into()
                };
            }
            Err(e) => self.status = format!("not saved: {e}"),
        }
    }

    fn restart_daemon(&mut self) {
        let result = Command::new("systemctl")
            .args(["--user", "restart", "ptw"])
            .status();
        self.status = match result {
            Ok(s) if s.success() => "daemon restarted".into(),
            Ok(s) => format!(
                "systemctl --user restart ptw failed ({s}); is the unit installed? run `ptw setup`"
            ),
            Err(e) => format!("cannot run systemctl: {e}"),
        };
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Hotkey");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.config.hotkey);
                    match self.hotkey_problem() {
                        Some(problem) => ui.colored_label(egui::Color32::RED, problem),
                        None => ui.label("hold to talk"),
                    };
                });
                ui.small("Keys joined with +, e.g. Alt+z, RightCtrl, Ctrl+Shift+Space. Any other key pressed during a hold cancels it.");
                ui.add_space(12.0);

                ui.heading("Custom words");
                ui.small("Names and jargon the recognizer gets wrong, one per line. Phrases are fine.");
                ui.add(egui::TextEdit::multiline(&mut self.custom_words_text).desired_rows(8).desired_width(f32::INFINITY));
                ui.add_space(12.0);

                ui.heading("Engine");
                egui::ComboBox::from_label("engine").selected_text(&self.config.engine.kind).show_ui(ui, |ui| {
                    for kind in engines::available() {
                        ui.selectable_value(&mut self.config.engine.kind, kind.to_string(), kind);
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("model");
                    ui.text_edit_singleline(&mut self.config.engine.model);
                });
                egui::ComboBox::from_label("lookahead")
                    .selected_text(format!("{} ms", self.config.engine.lookahead_ms))
                    .show_ui(ui, |ui| {
                        for ms in LOOKAHEADS_MS {
                            ui.selectable_value(&mut self.config.engine.lookahead_ms, ms, format!("{ms} ms"));
                        }
                    });
                ui.small("Shorter lookahead types sooner; longer is more accurate.");
                ui.horizontal(|ui| {
                    ui.label("threads (0 = automatic)");
                    ui.add(egui::DragValue::new(&mut self.config.engine.threads).range(0..=64));
                });
                ui.add_space(12.0);

                ui.heading("Audio");
                let mic_label = if self.config.audio.device.is_empty() { "default".to_string() } else { self.config.audio.device.clone() };
                egui::ComboBox::from_label("microphone").selected_text(mic_label).show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.config.audio.device, String::new(), "default");
                    for name in &self.microphones {
                        ui.selectable_value(&mut self.config.audio.device, name.clone(), name);
                    }
                });
                ui.checkbox(&mut self.config.audio.cues, "play a cue when a hold starts and ends");
                ui.add_space(12.0);

                ui.heading("Typing");
                ui.radio_value(&mut self.config.typist.backend, TypistBackend::Portal, "desktop portal (GNOME, KDE; asks once)");
                ui.radio_value(&mut self.config.typist.backend, TypistBackend::Uinput, "uinput virtual keyboard (needs `ptw setup`; US layout)");
                ui.add_space(16.0);

                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        self.save();
                    }
                    if ui.button("Restart daemon").clicked() {
                        self.restart_daemon();
                    }
                    ui.label(&self.status);
                });
                ui.small(format!("{}", self.path.display()));
        });
    }
}

pub fn run(config_path: &Path) -> anyhow::Result<()> {
    let app = App::new(config_path.to_path_buf())?;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([520.0, 640.0])
            .with_title("ptw settings"),
        ..Default::default()
    };
    eframe::run_native("ptw settings", options, Box::new(|_cc| Ok(Box::new(app))))
        .map_err(|e| anyhow::anyhow!("{e}"))
}
