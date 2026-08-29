//! Key names for the config file, mapped to Linux input event codes.
//!
//! The numbers are the `KEY_*` constants from `linux/input-event-codes.h`.
//! Only keys someone would plausibly put in a hotkey are listed.

use std::fmt;

/// A Linux input event key code.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct KeyCode(pub u16);

impl fmt::Display for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match key_name(*self) {
            Some(name) => f.write_str(name),
            None => write!(f, "key{}", self.0),
        }
    }
}

pub const LEFT_CTRL: KeyCode = KeyCode(29);
pub const LEFT_SHIFT: KeyCode = KeyCode(42);
pub const RIGHT_SHIFT: KeyCode = KeyCode(54);
pub const LEFT_ALT: KeyCode = KeyCode(56);
pub const RIGHT_CTRL: KeyCode = KeyCode(97);
pub const RIGHT_ALT: KeyCode = KeyCode(100);
pub const LEFT_SUPER: KeyCode = KeyCode(125);
pub const RIGHT_SUPER: KeyCode = KeyCode(126);

const NAMES: &[(&str, u16)] = &[
    ("Esc", 1),
    ("1", 2),
    ("2", 3),
    ("3", 4),
    ("4", 5),
    ("5", 6),
    ("6", 7),
    ("7", 8),
    ("8", 9),
    ("9", 10),
    ("0", 11),
    ("Minus", 12),
    ("Equal", 13),
    ("Backspace", 14),
    ("Tab", 15),
    ("q", 16),
    ("w", 17),
    ("e", 18),
    ("r", 19),
    ("t", 20),
    ("y", 21),
    ("u", 22),
    ("i", 23),
    ("o", 24),
    ("p", 25),
    ("LeftBrace", 26),
    ("RightBrace", 27),
    ("Enter", 28),
    ("LeftCtrl", 29),
    ("a", 30),
    ("s", 31),
    ("d", 32),
    ("f", 33),
    ("g", 34),
    ("h", 35),
    ("j", 36),
    ("k", 37),
    ("l", 38),
    ("Semicolon", 39),
    ("Apostrophe", 40),
    ("Grave", 41),
    ("LeftShift", 42),
    ("Backslash", 43),
    ("z", 44),
    ("x", 45),
    ("c", 46),
    ("v", 47),
    ("b", 48),
    ("n", 49),
    ("m", 50),
    ("Comma", 51),
    ("Dot", 52),
    ("Slash", 53),
    ("RightShift", 54),
    ("LeftAlt", 56),
    ("Space", 57),
    ("CapsLock", 58),
    ("F1", 59),
    ("F2", 60),
    ("F3", 61),
    ("F4", 62),
    ("F5", 63),
    ("F6", 64),
    ("F7", 65),
    ("F8", 66),
    ("F9", 67),
    ("F10", 68),
    ("NumLock", 69),
    ("ScrollLock", 70),
    ("F11", 87),
    ("F12", 88),
    ("RightCtrl", 97),
    ("RightAlt", 100),
    ("Home", 102),
    ("Up", 103),
    ("PageUp", 104),
    ("Left", 105),
    ("Right", 106),
    ("End", 107),
    ("Down", 108),
    ("PageDown", 109),
    ("Insert", 110),
    ("Delete", 111),
    ("Pause", 119),
    ("LeftSuper", 125),
    ("RightSuper", 126),
    ("Menu", 127),
    ("F13", 183),
    ("F14", 184),
    ("F15", 185),
    ("F16", 186),
    ("F17", 187),
    ("F18", 188),
    ("F19", 189),
    ("F20", 190),
    ("F21", 191),
    ("F22", 192),
    ("F23", 193),
    ("F24", 194),
];

/// Punctuation spellings people type in a config: `\` for Backslash, and so on.
const SYMBOL_ALIASES: &[(&str, &str)] = &[
    ("\\", "Backslash"),
    ("-", "Minus"),
    ("=", "Equal"),
    ("[", "LeftBrace"),
    ("]", "RightBrace"),
    (";", "Semicolon"),
    ("'", "Apostrophe"),
    ("`", "Grave"),
    (",", "Comma"),
    (".", "Dot"),
    ("/", "Slash"),
    ("Escape", "Esc"),
    ("Return", "Enter"),
    ("LeftMeta", "LeftSuper"),
    ("RightMeta", "RightSuper"),
    ("LeftWin", "LeftSuper"),
    ("RightWin", "RightSuper"),
    ("LeftControl", "LeftCtrl"),
    ("RightControl", "RightCtrl"),
    ("AltGr", "RightAlt"),
];

/// Looks up a single key by its config name, case-insensitively.
///
/// Side-agnostic modifier names (`Alt`, `Ctrl`) are not single keys; see
/// [`crate::hotkey::Chord`] for those.
pub fn key_code(name: &str) -> Option<KeyCode> {
    let name = SYMBOL_ALIASES
        .iter()
        .find(|(alias, _)| alias.eq_ignore_ascii_case(name))
        .map_or(name, |(_, canonical)| canonical);
    NAMES
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
        .map(|(_, code)| KeyCode(*code))
}

/// The config name for a key code, if it is one we list.
pub fn key_name(code: KeyCode) -> Option<&'static str> {
    NAMES
        .iter()
        .find(|(_, candidate)| *candidate == code.0)
        .map(|(name, _)| *name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_case_insensitive_and_aliases_resolve() {
        assert_eq!(key_code("z"), Some(KeyCode(44)));
        assert_eq!(key_code("Z"), Some(KeyCode(44)));
        assert_eq!(key_code("leftalt"), Some(LEFT_ALT));
        assert_eq!(key_code("\\"), Some(KeyCode(43)));
        assert_eq!(key_code("AltGr"), Some(RIGHT_ALT));
        assert_eq!(key_code("Alt"), None);
        assert_eq!(key_code("nonsense"), None);
    }

    #[test]
    fn names_round_trip() {
        for (name, code) in NAMES {
            assert_eq!(key_name(KeyCode(*code)), Some(*name));
            assert_eq!(key_code(name), Some(KeyCode(*code)), "{name}");
        }
    }

    #[test]
    fn table_has_no_duplicate_names_or_codes() {
        for (i, (name, code)) in NAMES.iter().enumerate() {
            for (other_name, other_code) in &NAMES[i + 1..] {
                assert!(!name.eq_ignore_ascii_case(other_name), "{name}");
                assert_ne!(code, other_code, "{name} and {other_name}");
            }
        }
    }
}
