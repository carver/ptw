//! Which physical key types which character: the keyboard layout.
//!
//! Key names in the config are what the key *types* (`Alt+z`), and the
//! hotkey reader sees physical key codes. On a Dvorak layout the `z` is
//! under QWERTY's `/`, so the two only agree through a [`Layout`].

use std::collections::BTreeMap;

use crate::keys::{self, KeyCode};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layout {
    name: String,
    keys: BTreeMap<char, Vec<KeyCode>>,
}

impl Layout {
    /// `name` is for logs (`us+dvorak`). A character on several keys, like
    /// digits with a numpad, lists them all.
    pub fn new(name: impl Into<String>, keys: impl IntoIterator<Item = (char, KeyCode)>) -> Self {
        let mut map: BTreeMap<char, Vec<KeyCode>> = BTreeMap::new();
        for (ch, code) in keys {
            let codes = map.entry(ch).or_default();
            if !codes.contains(&code) {
                codes.push(code);
            }
        }
        Self {
            name: name.into(),
            keys: map,
        }
    }

    pub fn qwerty() -> Self {
        Self::new("us", keys::qwerty_chars())
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// The keys that type `ch` (lowercase; the layout's unshifted level).
    pub fn codes(&self, ch: char) -> Option<&[KeyCode]> {
        self.keys.get(&ch).map(Vec::as_slice)
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::qwerty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qwerty_maps_characters_to_their_keys() {
        let layout = Layout::qwerty();
        assert_eq!(layout.codes('z'), Some(&[KeyCode(44)][..]));
        assert_eq!(layout.codes('Z'), None);
    }

    #[test]
    fn a_character_on_two_keys_lists_both_once() {
        let layout = Layout::new(
            "test",
            [('1', KeyCode(2)), ('1', KeyCode(79)), ('1', KeyCode(2))],
        );
        assert_eq!(layout.codes('1'), Some(&[KeyCode(2), KeyCode(79)][..]));
    }
}
