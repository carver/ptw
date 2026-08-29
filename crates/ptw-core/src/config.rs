//! The config file: TOML at `~/.config/ptw/config.toml`, the source of
//! truth that the settings dialog edits and the daemon watches.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::hotkey::Chord;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// The Hotkey chord, e.g. `Alt+z` or `RightCtrl`.
    pub hotkey: String,
    /// Custom words, one entry per word or phrase.
    pub custom_words: Vec<String>,
    pub engine: EngineConfig,
    pub audio: AudioConfig,
    pub typist: TypistConfig,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct EngineConfig {
    /// Which Engine to load. The daemon lists the ones it was built with.
    pub kind: String,
    /// Model directory or file, relative to the models directory unless absolute.
    pub model: String,
    /// Lookahead in milliseconds; which values exist depends on the Model.
    pub lookahead_ms: u32,
    /// Threads for recognition. 2 measured fastest on an Alder Lake laptop
    /// (ADR 0005); 0 lets the Engine decide.
    pub threads: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AudioConfig {
    /// Input device name as the audio host reports it; empty for the default.
    pub device: String,
    /// Play a short cue when a Hold starts and ends.
    pub cues: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TypistConfig {
    pub backend: TypistBackend,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypistBackend {
    /// XDG RemoteDesktop portal (ADR 0004).
    Portal,
    /// A uinput virtual keyboard; needs the udev rule from `ptw setup`.
    Uinput,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hotkey: "Alt+z".to_string(),
            custom_words: Vec::new(),
            engine: EngineConfig::default(),
            audio: AudioConfig::default(),
            typist: TypistConfig::default(),
        }
    }
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            kind: "transcribe".to_string(),
            model: "nemotron-speech-streaming-en-0.6b-Q8_0.gguf".to_string(),
            lookahead_ms: 560,
            threads: 2,
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            device: String::new(),
            cues: true,
        }
    }
}

impl Default for TypistConfig {
    fn default() -> Self {
        Self {
            backend: TypistBackend::Portal,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("cannot read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot write {path}: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("{path} is not valid: {source}")]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("hotkey in {path}: {source}")]
    Hotkey {
        path: PathBuf,
        source: crate::hotkey::ChordParseError,
    },
}

impl Config {
    pub fn chord(&self) -> Result<Chord, crate::hotkey::ChordParseError> {
        self.hotkey.parse()
    }

    /// Reads the file, or returns the defaults when it does not exist.
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(source) => {
                return Err(ConfigError::Read {
                    path: path.to_path_buf(),
                    source,
                });
            }
        };
        let config: Self = toml::from_str(&text).map_err(|source| ConfigError::Parse {
            path: path.to_path_buf(),
            source,
        })?;
        config.chord().map_err(|source| ConfigError::Hotkey {
            path: path.to_path_buf(),
            source,
        })?;
        Ok(config)
    }

    /// Writes the file, creating parent directories. The write is atomic:
    /// a daemon watching the file never sees a half-written one.
    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        let write = |source| ConfigError::Write {
            path: path.to_path_buf(),
            source,
        };
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(write)?;
        }
        let text = toml::to_string_pretty(self).expect("config serializes");
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, text).map_err(write)?;
        std::fs::rename(&tmp, path).map_err(write)
    }
}

/// `$XDG_CONFIG_HOME/ptw` or `~/.config/ptw`.
pub fn config_dir() -> PathBuf {
    xdg_dir("XDG_CONFIG_HOME", ".config").join("ptw")
}

/// `$XDG_DATA_HOME/ptw` or `~/.local/share/ptw`; models live under `models/`.
pub fn data_dir() -> PathBuf {
    xdg_dir("XDG_DATA_HOME", ".local/share").join("ptw")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

fn xdg_dir(var: &str, fallback: &str) -> PathBuf {
    match std::env::var_os(var).filter(|v| !v.is_empty()) {
        Some(dir) => PathBuf::from(dir),
        None => {
            let home = std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/"));
            home.join(fallback)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("ptw-config-test-{}-{}", std::process::id(), name));
        dir.join("config.toml")
    }

    #[test]
    fn missing_file_means_defaults() {
        let config = Config::load(Path::new("/nonexistent/ptw/config.toml")).unwrap();
        assert_eq!(config, Config::default());
        assert_eq!(config.chord().unwrap().to_string(), "Alt+z");
    }

    #[test]
    fn round_trips_through_toml() {
        let path = temp_path("roundtrip");
        let config = Config {
            hotkey: "RightCtrl".into(),
            custom_words: vec!["Caitlyn".into(), "Jason Carver".into()],
            engine: EngineConfig {
                lookahead_ms: 160,
                ..Default::default()
            },
            typist: TypistConfig {
                backend: TypistBackend::Uinput,
            },
            ..Default::default()
        };
        config.save(&path).unwrap();
        assert_eq!(Config::load(&path).unwrap(), config);
        assert!(!path.with_extension("toml.tmp").exists());
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn partial_files_fill_in_defaults_and_unknown_keys_are_errors() {
        let path = temp_path("partial");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            &path,
            "custom_words = [\"Caitlyn\"]\n[engine]\nlookahead_ms = 160\n",
        )
        .unwrap();
        let config = Config::load(&path).unwrap();
        assert_eq!(config.custom_words, vec!["Caitlyn".to_string()]);
        assert_eq!(config.engine.lookahead_ms, 160);
        assert_eq!(config.hotkey, "Alt+z");

        std::fs::write(&path, "hotkey = \"Alt+z\"\nbanana = 1\n").unwrap();
        assert!(matches!(
            Config::load(&path),
            Err(ConfigError::Parse { .. })
        ));

        std::fs::write(&path, "hotkey = \"Alt+banana\"\n").unwrap();
        assert!(matches!(
            Config::load(&path),
            Err(ConfigError::Hotkey { .. })
        ));
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
