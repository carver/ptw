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
    layout: ptw_core::layout::Layout,
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
            layout: crate::layout::detect(),
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
        Chord::parse(&self.config.hotkey, &self.layout)
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
            egui::Frame::new()
                .inner_margin(egui::Margin::same(20))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(12.0, 8.0);
                    self.hotkey_section(ui);
                    self.custom_words_section(ui);
                    self.engine_section(ui);
                    self.audio_section(ui);
                    self.typing_section(ui);
                    self.save_row(ui);
                });
        });
    }
}

fn section(ui: &mut egui::Ui, title: &str, hint: &str) {
    ui.add_space(8.0);
    ui.heading(title);
    if !hint.is_empty() {
        ui.weak(hint);
    }
    ui.add_space(4.0);
}

fn grid(ui: &mut egui::Ui, id: &str, add_rows: impl FnOnce(&mut egui::Ui)) {
    egui::Grid::new(id)
        .num_columns(2)
        .spacing([24.0, 10.0])
        .min_col_width(110.0)
        .show(ui, add_rows);
}

impl App {
    fn hotkey_section(&mut self, ui: &mut egui::Ui) {
        section(
            ui,
            "Hotkey",
            "Hold it to talk. Keys joined with +, e.g. Alt+z, RightCtrl, Ctrl+Shift+Space. Any other key pressed during a hold cancels it.",
        );
        grid(ui, "hotkey", |ui| {
            ui.label("Hotkey");
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.config.hotkey).desired_width(160.0));
                match (
                    self.hotkey_problem(),
                    Chord::parse(&self.config.hotkey, &self.layout),
                ) {
                    (Some(problem), _) => {
                        ui.colored_label(ui.visuals().error_fg_color, problem);
                    }
                    (None, Ok(chord)) if chord.physical() != chord.to_string() => {
                        ui.weak(format!(
                            "physical {} on {}",
                            chord.physical(),
                            self.layout.name()
                        ));
                    }
                    _ => {}
                }
            });
            ui.end_row();
        });
    }

    fn custom_words_section(&mut self, ui: &mut egui::Ui) {
        section(
            ui,
            "Custom words",
            "Names and jargon the recognizer gets wrong, one per line. Phrases are fine.",
        );
        ui.add(
            egui::TextEdit::multiline(&mut self.custom_words_text)
                .desired_rows(8)
                .desired_width(f32::INFINITY),
        );
    }

    fn engine_section(&mut self, ui: &mut egui::Ui) {
        section(ui, "Engine", "Changes here need a daemon restart.");
        grid(ui, "engine", |ui| {
            ui.label("Engine");
            egui::ComboBox::from_id_salt("engine")
                .selected_text(&self.config.engine.kind)
                .show_ui(ui, |ui| {
                    for kind in engines::available() {
                        ui.selectable_value(&mut self.config.engine.kind, kind.to_string(), kind);
                    }
                });
            ui.end_row();

            ui.label("Model");
            ui.add(
                egui::TextEdit::singleline(&mut self.config.engine.model)
                    .desired_width(f32::INFINITY),
            );
            ui.end_row();

            ui.label("Lookahead");
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("lookahead")
                    .selected_text(format!("{} ms", self.config.engine.lookahead_ms))
                    .show_ui(ui, |ui| {
                        for ms in LOOKAHEADS_MS {
                            ui.selectable_value(
                                &mut self.config.engine.lookahead_ms,
                                ms,
                                format!("{ms} ms"),
                            );
                        }
                    });
                ui.weak("shorter types sooner, longer is more accurate");
            });
            ui.end_row();

            ui.label("Threads");
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut self.config.engine.threads).range(0..=64));
                ui.weak("0 = automatic");
            });
            ui.end_row();
        });
    }

    fn audio_section(&mut self, ui: &mut egui::Ui) {
        section(ui, "Audio", "");
        grid(ui, "audio", |ui| {
            ui.label("Microphone");
            let mic_label = if self.config.audio.device.is_empty() {
                "default".to_string()
            } else {
                self.config.audio.device.clone()
            };
            egui::ComboBox::from_id_salt("microphone")
                .selected_text(mic_label)
                .width(300.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.config.audio.device, String::new(), "default");
                    for name in &self.microphones {
                        ui.selectable_value(&mut self.config.audio.device, name.clone(), name);
                    }
                });
            ui.end_row();

            ui.label("Cues");
            ui.checkbox(
                &mut self.config.audio.cues,
                "play a sound when a hold starts and ends",
            );
            ui.end_row();
        });
    }

    fn typing_section(&mut self, ui: &mut egui::Ui) {
        section(
            ui,
            "Typing",
            "How text reaches the focused app. Needs a daemon restart.",
        );
        ui.radio_value(
            &mut self.config.typist.backend,
            TypistBackend::Portal,
            "Desktop portal (GNOME, KDE; asks for permission once)",
        );
        ui.radio_value(
            &mut self.config.typist.backend,
            TypistBackend::Uinput,
            "uinput virtual keyboard (any compositor; US layout only)",
        );
    }

    fn save_row(&mut self, ui: &mut egui::Ui) {
        ui.add_space(16.0);
        ui.separator();
        ui.horizontal(|ui| {
            ui.weak(self.path.display().to_string());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let dirty = self.config != self.saved
                    || self
                        .custom_words_text
                        .lines()
                        .map(str::trim)
                        .filter(|l| !l.is_empty())
                        .ne(self.saved.custom_words.iter().map(String::as_str));
                if ui.add_enabled(dirty, egui::Button::new("Save")).clicked() {
                    self.save();
                }
                if ui.button("Restart daemon").clicked() {
                    self.restart_daemon();
                }
            });
        });
        if !self.status.is_empty() {
            ui.label(&self.status);
        }
    }
}

pub fn run(config_path: &Path) -> anyhow::Result<()> {
    let app = App::new(config_path.to_path_buf())?;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([560.0, 720.0])
            .with_min_inner_size([420.0, 400.0])
            .with_title("ptw settings"),
        ..Default::default()
    };
    eframe::run_native(
        "ptw settings",
        options,
        Box::new(|cc| {
            cc.egui_ctx.all_styles_mut(|style| {
                for font in style.text_styles.values_mut() {
                    font.size *= 1.15;
                }
            });
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| anyhow::anyhow!("{e}"))
}
