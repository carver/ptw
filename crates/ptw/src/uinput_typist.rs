//! Types through a uinput virtual keyboard: works on any compositor, needs
//! `/dev/uinput` access, and only knows the US layout.

use evdev::uinput::VirtualDevice;
use evdev::{AttributeSet, EventType, InputEvent, KeyCode};
use ptw_core::typist::{Typist, TypistError};

use crate::hotkey_source::OWN_DEVICE_NAME;

pub struct UinputTypist {
    device: VirtualDevice,
}

impl UinputTypist {
    pub fn open() -> anyhow::Result<Self> {
        let mut keys = AttributeSet::<KeyCode>::new();
        for code in 1..=127u16 {
            keys.insert(KeyCode(code));
        }
        let device = VirtualDevice::builder()?
            .name(OWN_DEVICE_NAME)
            .with_keys(&keys)?
            .build()?;
        // The compositor needs a moment to pick up a new device.
        std::thread::sleep(std::time::Duration::from_millis(300));
        Ok(Self { device })
    }

    fn tap(&mut self, key: KeyCode, shift: bool) -> std::io::Result<()> {
        let mut events = Vec::with_capacity(4);
        if shift {
            events.push(InputEvent::new(
                EventType::KEY.0,
                KeyCode::KEY_LEFTSHIFT.0,
                1,
            ));
        }
        events.push(InputEvent::new(EventType::KEY.0, key.0, 1));
        events.push(InputEvent::new(EventType::KEY.0, key.0, 0));
        if shift {
            events.push(InputEvent::new(
                EventType::KEY.0,
                KeyCode::KEY_LEFTSHIFT.0,
                0,
            ));
        }
        self.device.emit(&events)
    }
}

/// US layout: the key and whether Shift is held.
fn us_key(ch: char) -> Option<(KeyCode, bool)> {
    use KeyCode as K;
    let unshifted = |c| {
        match c {
            'a' => K::KEY_A,
            'b' => K::KEY_B,
            'c' => K::KEY_C,
            'd' => K::KEY_D,
            'e' => K::KEY_E,
            'f' => K::KEY_F,
            'g' => K::KEY_G,
            'h' => K::KEY_H,
            'i' => K::KEY_I,
            'j' => K::KEY_J,
            'k' => K::KEY_K,
            'l' => K::KEY_L,
            'm' => K::KEY_M,
            'n' => K::KEY_N,
            'o' => K::KEY_O,
            'p' => K::KEY_P,
            'q' => K::KEY_Q,
            'r' => K::KEY_R,
            's' => K::KEY_S,
            't' => K::KEY_T,
            'u' => K::KEY_U,
            'v' => K::KEY_V,
            'w' => K::KEY_W,
            'x' => K::KEY_X,
            'y' => K::KEY_Y,
            'z' => K::KEY_Z,
            '1' => K::KEY_1,
            '2' => K::KEY_2,
            '3' => K::KEY_3,
            '4' => K::KEY_4,
            '5' => K::KEY_5,
            '6' => K::KEY_6,
            '7' => K::KEY_7,
            '8' => K::KEY_8,
            '9' => K::KEY_9,
            '0' => K::KEY_0,
            ' ' => K::KEY_SPACE,
            '\n' => K::KEY_ENTER,
            '\t' => K::KEY_TAB,
            '-' => K::KEY_MINUS,
            '=' => K::KEY_EQUAL,
            '[' => K::KEY_LEFTBRACE,
            ']' => K::KEY_RIGHTBRACE,
            ';' => K::KEY_SEMICOLON,
            '\'' => K::KEY_APOSTROPHE,
            '`' => K::KEY_GRAVE,
            '\\' => K::KEY_BACKSLASH,
            ',' => K::KEY_COMMA,
            '.' => K::KEY_DOT,
            '/' => K::KEY_SLASH,
            _ => return None,
        }
        .into()
    };
    if let Some(key) = unshifted(ch) {
        return Some((key, false));
    }
    let shifted = match ch {
        'A'..='Z' => return unshifted(ch.to_ascii_lowercase()).map(|k| (k, true)),
        '!' => K::KEY_1,
        '@' => K::KEY_2,
        '#' => K::KEY_3,
        '$' => K::KEY_4,
        '%' => K::KEY_5,
        '^' => K::KEY_6,
        '&' => K::KEY_7,
        '*' => K::KEY_8,
        '(' => K::KEY_9,
        ')' => K::KEY_0,
        '_' => K::KEY_MINUS,
        '+' => K::KEY_EQUAL,
        '{' => K::KEY_LEFTBRACE,
        '}' => K::KEY_RIGHTBRACE,
        ':' => K::KEY_SEMICOLON,
        '"' => K::KEY_APOSTROPHE,
        '~' => K::KEY_GRAVE,
        '|' => K::KEY_BACKSLASH,
        '<' => K::KEY_COMMA,
        '>' => K::KEY_DOT,
        '?' => K::KEY_SLASH,
        _ => return None,
    };
    Some((shifted, true))
}

/// Characters the US layout cannot type get an ASCII stand-in or are dropped.
fn ascii_stand_in(ch: char) -> Option<char> {
    match ch {
        '\u{2018}' | '\u{2019}' => Some('\''),
        '\u{201c}' | '\u{201d}' => Some('"'),
        '\u{2013}' | '\u{2014}' => Some('-'),
        '\u{2026}' => Some('.'),
        _ => None,
    }
}

impl Typist for UinputTypist {
    fn type_text(&mut self, text: &str) -> Result<(), TypistError> {
        for ch in text.chars() {
            let Some((key, shift)) = us_key(ch).or_else(|| ascii_stand_in(ch).and_then(us_key))
            else {
                continue;
            };
            self.tap(key, shift)
                .map_err(|e| TypistError::Backend(e.to_string()))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_printable_ascii() {
        for ch in (32u8..127).map(char::from) {
            assert!(us_key(ch).is_some(), "{ch:?}");
        }
        assert_eq!(us_key('A'), Some((KeyCode::KEY_A, true)));
        assert_eq!(us_key('?'), Some((KeyCode::KEY_SLASH, true)));
        assert_eq!(us_key('\u{e9}'), None);
    }
}
