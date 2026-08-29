//! The Hotkey: a chord of keys whose Hold starts a Dictation.
//!
//! [`HotkeyMachine`] turns raw key presses and releases into
//! [`HotkeyEvent`]s. It never sees audio or text.

use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;

use crate::keys::{self, KeyCode};
use crate::layout::Layout;

/// One member of a chord: a key, or any of several keys (`Alt` is either Alt).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChordKey {
    name: String,
    codes: Vec<KeyCode>,
}

impl ChordKey {
    fn parse(name: &str, layout: &Layout) -> Result<Self, ChordParseError> {
        let side_agnostic: &[(&str, [KeyCode; 2])] = &[
            ("Alt", [keys::LEFT_ALT, keys::RIGHT_ALT]),
            ("Ctrl", [keys::LEFT_CTRL, keys::RIGHT_CTRL]),
            ("Control", [keys::LEFT_CTRL, keys::RIGHT_CTRL]),
            ("Shift", [keys::LEFT_SHIFT, keys::RIGHT_SHIFT]),
            ("Super", [keys::LEFT_SUPER, keys::RIGHT_SUPER]),
            ("Meta", [keys::LEFT_SUPER, keys::RIGHT_SUPER]),
            ("Win", [keys::LEFT_SUPER, keys::RIGHT_SUPER]),
        ];
        if let Some((canonical, codes)) = side_agnostic
            .iter()
            .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
        {
            return Ok(Self {
                name: (*canonical).to_string(),
                codes: codes.to_vec(),
            });
        }
        let mut chars = name.chars();
        if let (Some(ch), None) = (chars.next(), chars.next())
            && let Some(codes) = layout.codes(ch.to_ascii_lowercase())
        {
            return Ok(Self {
                name: ch.to_ascii_lowercase().to_string(),
                codes: codes.to_vec(),
            });
        }
        let code =
            keys::key_code(name).ok_or_else(|| ChordParseError::UnknownKey(name.to_string()))?;
        Ok(Self {
            name: keys::key_name(code).unwrap_or(name).to_string(),
            codes: vec![code],
        })
    }

    fn matches(&self, code: KeyCode) -> bool {
        self.codes.contains(&code)
    }
}

/// The keys that must all be held for the Hotkey to be down, as written in
/// the config: `Alt+z`, `RightCtrl`, `Ctrl+Shift+Space`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chord {
    keys: Vec<ChordKey>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ChordParseError {
    #[error("hotkey is empty")]
    Empty,
    #[error("unknown key `{0}`")]
    UnknownKey(String),
    #[error("key `{0}` appears twice")]
    Duplicate(String),
}

/// Parses with a QWERTY layout; see [`Chord::parse`] for the user's.
impl FromStr for Chord {
    type Err = ChordParseError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::parse(text, &Layout::qwerty())
    }
}

impl Chord {
    /// Single-character names (`z`, `/`) mean the key that types that
    /// character under `layout`; everything else names a physical key.
    pub fn parse(text: &str, layout: &Layout) -> Result<Self, ChordParseError> {
        let mut keys = Vec::new();
        for part in text.split('+').map(str::trim) {
            if part.is_empty() {
                return Err(ChordParseError::Empty);
            }
            let key = ChordKey::parse(part, layout)?;
            if keys
                .iter()
                .any(|existing: &ChordKey| existing.codes.iter().any(|c| key.matches(*c)))
            {
                return Err(ChordParseError::Duplicate(key.name));
            }
            keys.push(key);
        }
        Ok(Self { keys })
    }
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<&str> = self.keys.iter().map(|k| k.name.as_str()).collect();
        f.write_str(&names.join("+"))
    }
}

impl Chord {
    pub fn is_member(&self, code: KeyCode) -> bool {
        self.keys.iter().any(|k| k.matches(code))
    }

    fn is_satisfied_by(&self, held: &BTreeSet<KeyCode>) -> bool {
        self.keys.iter().all(|k| held.iter().any(|c| k.matches(*c)))
    }

    /// Every key code that could take part in this chord.
    pub fn codes(&self) -> impl Iterator<Item = KeyCode> + '_ {
        self.keys.iter().flat_map(|k| k.codes.iter().copied())
    }

    /// The physical keys behind the chord, for logs: `Alt+z` on Dvorak is
    /// `Alt+Slash`.
    pub fn physical(&self) -> String {
        let names: Vec<String> = self
            .keys
            .iter()
            .map(|k| {
                if k.codes.len() > 1 && k.codes.iter().all(|c| keys::is_modifier(*c)) {
                    k.name.clone()
                } else {
                    k.codes
                        .iter()
                        .map(|c| c.to_string())
                        .collect::<Vec<_>>()
                        .join("/")
                }
            })
            .collect();
        names.join("+")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyAction {
    Press,
    Release,
    /// Autorepeat. Ignored, but callers should not have to filter it.
    Repeat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub action: KeyAction,
}

impl KeyEvent {
    pub fn press(code: KeyCode) -> Self {
        Self {
            code,
            action: KeyAction::Press,
        }
    }

    pub fn release(code: KeyCode) -> Self {
        Self {
            code,
            action: KeyAction::Release,
        }
    }
}

/// What a key event meant for the Dictation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HotkeyEvent {
    /// The chord is down: start capturing.
    Start,
    /// A chord key came up: stop capturing and Flush.
    Stop,
    /// Another key was pressed during the Hold: discard the Dictation.
    Abort,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Idle,
    Holding,
    /// Aborted; stays here until every chord key is up, so a chord that is
    /// still held cannot restart a Dictation by itself.
    Latched,
}

/// Tracks held keys and reports when a Hold starts, stops, or is aborted.
#[derive(Clone, Debug)]
pub struct HotkeyMachine {
    chord: Chord,
    held: BTreeSet<KeyCode>,
    state: State,
}

impl HotkeyMachine {
    pub fn new(chord: Chord) -> Self {
        Self {
            chord,
            held: BTreeSet::new(),
            state: State::Idle,
        }
    }

    pub fn chord(&self) -> &Chord {
        &self.chord
    }

    pub fn is_holding(&self) -> bool {
        self.state == State::Holding
    }

    /// No chord key is doing anything: not holding, not latched.
    pub fn is_idle(&self) -> bool {
        self.state == State::Idle
    }

    /// Whether a modifier key is down. Text typed now would reach the
    /// focused app as shortcuts (Alt+Space opens GNOME's window menu), so
    /// the Typist must wait until this is false.
    pub fn modifiers_held(&self) -> bool {
        self.held.iter().any(|c| keys::is_modifier(*c))
    }

    pub fn on_key(&mut self, event: KeyEvent) -> Option<HotkeyEvent> {
        match event.action {
            KeyAction::Repeat => None,
            KeyAction::Press => {
                self.held.insert(event.code);
                match self.state {
                    State::Idle if self.chord.is_satisfied_by(&self.held) => {
                        self.state = State::Holding;
                        Some(HotkeyEvent::Start)
                    }
                    State::Holding if !self.chord.is_member(event.code) => {
                        self.state = State::Latched;
                        Some(HotkeyEvent::Abort)
                    }
                    _ => None,
                }
            }
            KeyAction::Release => {
                self.held.remove(&event.code);
                match self.state {
                    State::Holding if !self.chord.is_satisfied_by(&self.held) => {
                        self.state = State::Idle;
                        Some(HotkeyEvent::Stop)
                    }
                    State::Latched if !self.held.iter().any(|c| self.chord.is_member(*c)) => {
                        self.state = State::Idle;
                        None
                    }
                    _ => None,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::{LEFT_ALT, LEFT_SHIFT, RIGHT_ALT, RIGHT_CTRL, key_code};

    fn chord(text: &str) -> Chord {
        text.parse().unwrap()
    }

    fn drive(machine: &mut HotkeyMachine, events: &[KeyEvent]) -> Vec<Option<HotkeyEvent>> {
        events.iter().map(|e| machine.on_key(*e)).collect()
    }

    #[test]
    fn parses_and_prints_chords() {
        assert_eq!(chord("Alt+z").to_string(), "Alt+z");
        assert_eq!(chord("alt + Z").to_string(), "Alt+z");
        assert_eq!(chord("RightCtrl").to_string(), "RightCtrl");
        assert_eq!(chord("ctrl+shift+space").to_string(), "Ctrl+Shift+Space");
        assert_eq!(chord("Alt+\\").to_string(), "Alt+\\");
        assert_eq!(chord("Alt+\\").physical(), "Alt+Backslash");
    }

    #[test]
    fn single_characters_follow_the_layout() {
        let dvorak = Layout::new(
            "us+dvorak",
            [
                ('z', key_code("Slash").unwrap()),
                (';', key_code("z").unwrap()),
            ],
        );
        let on_dvorak = Chord::parse("Alt+z", &dvorak).unwrap();
        assert_eq!(on_dvorak.to_string(), "Alt+z");
        assert_eq!(on_dvorak.physical(), "Alt+Slash");
        assert_eq!(Chord::parse("Alt+;", &dvorak).unwrap().physical(), "Alt+z");
        assert_eq!(
            Chord::parse("RightCtrl", &dvorak).unwrap().physical(),
            "RightCtrl"
        );
        assert_eq!(chord("Alt+z").physical(), "Alt+z");
    }

    #[test]
    fn reports_held_modifiers() {
        let z = key_code("z").unwrap();
        let mut m = HotkeyMachine::new(chord("Alt+z"));
        assert!(!m.modifiers_held());
        m.on_key(KeyEvent::press(LEFT_ALT));
        m.on_key(KeyEvent::press(z));
        assert!(m.modifiers_held());
        m.on_key(KeyEvent::release(z));
        assert!(m.modifiers_held());
        m.on_key(KeyEvent::release(LEFT_ALT));
        assert!(!m.modifiers_held());
    }

    #[test]
    fn rejects_bad_chords() {
        assert_eq!("".parse::<Chord>(), Err(ChordParseError::Empty));
        assert_eq!("Alt+".parse::<Chord>(), Err(ChordParseError::Empty));
        assert_eq!(
            "Alt+banana".parse::<Chord>(),
            Err(ChordParseError::UnknownKey("banana".into()))
        );
        assert_eq!(
            "Alt+LeftAlt".parse::<Chord>(),
            Err(ChordParseError::Duplicate("LeftAlt".into()))
        );
        assert_eq!(
            "z+z".parse::<Chord>(),
            Err(ChordParseError::Duplicate("z".into()))
        );
    }

    #[test]
    fn hold_and_release_in_either_order() {
        let z = key_code("z").unwrap();
        let mut m = HotkeyMachine::new(chord("Alt+z"));
        let got = drive(
            &mut m,
            &[
                KeyEvent::press(LEFT_ALT),
                KeyEvent::press(z),
                KeyEvent {
                    code: z,
                    action: KeyAction::Repeat,
                },
                KeyEvent::release(z),
                KeyEvent::release(LEFT_ALT),
            ],
        );
        assert_eq!(
            got,
            vec![
                None,
                Some(HotkeyEvent::Start),
                None,
                Some(HotkeyEvent::Stop),
                None
            ]
        );

        let got = drive(
            &mut m,
            &[
                KeyEvent::press(z),
                KeyEvent::press(RIGHT_ALT),
                KeyEvent::release(RIGHT_ALT),
            ],
        );
        assert_eq!(
            got,
            vec![None, Some(HotkeyEvent::Start), Some(HotkeyEvent::Stop)]
        );
    }

    #[test]
    fn z_can_be_repressed_while_alt_stays_down() {
        let z = key_code("z").unwrap();
        let mut m = HotkeyMachine::new(chord("Alt+z"));
        let got = drive(
            &mut m,
            &[
                KeyEvent::press(LEFT_ALT),
                KeyEvent::press(z),
                KeyEvent::release(z),
                KeyEvent::press(z),
                KeyEvent::release(z),
            ],
        );
        assert_eq!(
            got,
            vec![
                None,
                Some(HotkeyEvent::Start),
                Some(HotkeyEvent::Stop),
                Some(HotkeyEvent::Start),
                Some(HotkeyEvent::Stop)
            ]
        );
    }

    #[test]
    fn another_key_during_the_hold_aborts_and_latches() {
        let z = key_code("z").unwrap();
        let x = key_code("x").unwrap();
        let mut m = HotkeyMachine::new(chord("Alt+z"));
        let got = drive(
            &mut m,
            &[
                KeyEvent::press(LEFT_ALT),
                KeyEvent::press(z),
                KeyEvent::press(x),
                KeyEvent::release(x),
                KeyEvent::release(z),
                KeyEvent::press(z),
                KeyEvent::release(z),
                KeyEvent::release(LEFT_ALT),
                KeyEvent::press(LEFT_ALT),
                KeyEvent::press(z),
            ],
        );
        assert_eq!(
            got,
            vec![
                None,
                Some(HotkeyEvent::Start),
                Some(HotkeyEvent::Abort),
                None,
                None,
                None,
                None,
                None,
                None,
                Some(HotkeyEvent::Start)
            ]
        );
    }

    #[test]
    fn keys_held_before_the_hold_do_not_abort_it() {
        let z = key_code("z").unwrap();
        let mut m = HotkeyMachine::new(chord("Alt+z"));
        let got = drive(
            &mut m,
            &[
                KeyEvent::press(LEFT_SHIFT),
                KeyEvent::press(LEFT_ALT),
                KeyEvent::press(z),
                KeyEvent::release(LEFT_SHIFT),
                KeyEvent::release(z),
            ],
        );
        assert_eq!(
            got,
            vec![
                None,
                None,
                Some(HotkeyEvent::Start),
                None,
                Some(HotkeyEvent::Stop)
            ]
        );
    }

    #[test]
    fn a_lone_modifier_works_as_a_hotkey_and_chords_with_it_abort() {
        let c = key_code("c").unwrap();
        let mut m = HotkeyMachine::new(chord("RightCtrl"));
        let got = drive(
            &mut m,
            &[KeyEvent::press(RIGHT_CTRL), KeyEvent::release(RIGHT_CTRL)],
        );
        assert_eq!(got, vec![Some(HotkeyEvent::Start), Some(HotkeyEvent::Stop)]);

        let got = drive(
            &mut m,
            &[
                KeyEvent::press(RIGHT_CTRL),
                KeyEvent::press(c),
                KeyEvent::release(c),
                KeyEvent::release(RIGHT_CTRL),
            ],
        );
        assert_eq!(
            got,
            vec![
                Some(HotkeyEvent::Start),
                Some(HotkeyEvent::Abort),
                None,
                None
            ]
        );
    }
}
