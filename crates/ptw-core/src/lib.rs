//! The parts of ptw that do not touch the desktop: hotkey chords, custom-word
//! correction, hold-back, and the traits the desktop crate implements.

pub mod config;
pub mod correction;
pub mod dictation;
pub mod engine;
pub mod hotkey;
pub mod keys;
pub mod layout;
pub mod proxy;
pub mod resample;
pub mod session;
pub mod typist;
