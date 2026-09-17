//! Taking a keyboard away from the compositor without stranding a key.
//!
//! Mutter counts every key across all input devices and delivers a press
//! only when that count goes from 0 to 1, a release only from 1 to 0.
//! libinput adds a check per device: a press for a key it already has
//! down is ignored. So when a grab starts while the compositor has a key
//! down, the release goes to the grabber and the compositor keeps the key
//! down on that device: dead for every app, and for the Typist, until the
//! grab ends and the key is pressed and released once more.
//!
//! The grab is therefore taken only while nothing is down, and checked
//! again afterwards. A press that landed between the check and the grab
//! is still down at the second check, because no finger leaves a key
//! within microseconds, and the grab is given back until it comes up.

use crate::keys::KeyCode;

/// A keyboard the compositor is reading, which we may take over.
pub trait Grabbable {
    type Error;

    /// Every key the kernel has down on this keyboard right now.
    fn keys_down(&mut self) -> Result<Vec<KeyCode>, Self::Error>;
    fn grab(&mut self) -> Result<(), Self::Error>;
    fn ungrab(&mut self) -> Result<(), Self::Error>;
}

/// Grabs `keyboard` once no key is down both before and after the grab.
/// `wait` is called with the keys still down each time the grab has to
/// wait; it should sleep a little.
pub fn grab_when_idle<K: Grabbable>(
    keyboard: &mut K,
    mut wait: impl FnMut(&[KeyCode]),
) -> Result<(), K::Error> {
    loop {
        let down = keyboard.keys_down()?;
        if !down.is_empty() {
            wait(&down);
            continue;
        }
        keyboard.grab()?;
        let down = keyboard.keys_down()?;
        if down.is_empty() {
            return Ok(());
        }
        keyboard.ungrab()?;
        wait(&down);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::{LEFT_ALT, key_code};
    use proptest::prelude::*;
    use std::collections::VecDeque;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Call {
        /// A key-state check, and whether anything was down.
        Check(bool),
        Grab,
        Ungrab,
    }

    /// Answers each key-state check from a script of snapshots.
    struct Scripted {
        snapshots: VecDeque<Vec<KeyCode>>,
        calls: Vec<Call>,
        fail_checks: bool,
    }

    impl Scripted {
        fn new(snapshots: &[&[KeyCode]]) -> Self {
            Self {
                snapshots: snapshots.iter().map(|s| s.to_vec()).collect(),
                calls: Vec::new(),
                fail_checks: false,
            }
        }
    }

    impl Grabbable for Scripted {
        type Error = &'static str;

        fn keys_down(&mut self) -> Result<Vec<KeyCode>, Self::Error> {
            if self.fail_checks {
                return Err("keyboard went away");
            }
            let down = self.snapshots.pop_front().expect("script ran out");
            self.calls.push(Call::Check(!down.is_empty()));
            Ok(down)
        }

        fn grab(&mut self) -> Result<(), Self::Error> {
            self.calls.push(Call::Grab);
            Ok(())
        }

        fn ungrab(&mut self) -> Result<(), Self::Error> {
            self.calls.push(Call::Ungrab);
            Ok(())
        }
    }

    fn z() -> KeyCode {
        key_code("Slash").unwrap()
    }

    #[test]
    fn grabs_at_once_when_nothing_is_down() {
        let mut kb = Scripted::new(&[&[], &[]]);
        let mut waits = 0;
        grab_when_idle(&mut kb, |_| waits += 1).unwrap();
        assert_eq!(
            kb.calls,
            vec![Call::Check(false), Call::Grab, Call::Check(false)]
        );
        assert_eq!(waits, 0);
    }

    #[test]
    fn waits_for_held_keys_and_reports_them() {
        let mut kb = Scripted::new(&[&[LEFT_ALT, z()], &[z()], &[], &[]]);
        let mut reported = Vec::new();
        grab_when_idle(&mut kb, |down| reported.push(down.to_vec())).unwrap();
        assert_eq!(reported, vec![vec![LEFT_ALT, z()], vec![z()]]);
        assert_eq!(
            kb.calls,
            vec![
                Call::Check(true),
                Call::Check(true),
                Call::Check(false),
                Call::Grab,
                Call::Check(false)
            ]
        );
    }

    #[test]
    fn a_press_that_slips_in_before_the_grab_gives_it_back() {
        let mut kb = Scripted::new(&[&[], &[z()], &[z()], &[], &[]]);
        let mut waits = 0;
        grab_when_idle(&mut kb, |_| waits += 1).unwrap();
        assert_eq!(
            kb.calls,
            vec![
                Call::Check(false),
                Call::Grab,
                Call::Check(true),
                Call::Ungrab,
                Call::Check(true),
                Call::Check(false),
                Call::Grab,
                Call::Check(false)
            ]
        );
        assert_eq!(waits, 2);
    }

    #[test]
    fn a_failing_keyboard_is_an_error_not_a_grab() {
        let mut kb = Scripted::new(&[]);
        kb.fail_checks = true;
        assert_eq!(grab_when_idle(&mut kb, |_| {}), Err("keyboard went away"));
        assert!(kb.calls.is_empty());
    }

    proptest! {
        /// Whatever the hand does, the grab that sticks is bracketed by two
        /// empty checks, and every other grab is handed straight back.
        #[test]
        fn every_kept_grab_saw_nothing_down_before_and_after(
            held in proptest::collection::vec(any::<bool>(), 0..24),
        ) {
            let snapshots: Vec<Vec<KeyCode>> = held
                .iter()
                .map(|&down| if down { vec![z()] } else { vec![] })
                .chain([vec![], vec![]])
                .collect();
            let borrowed: Vec<&[KeyCode]> = snapshots.iter().map(Vec::as_slice).collect();
            let mut kb = Scripted::new(&borrowed);
            let mut empty_waits = 0;
            grab_when_idle(&mut kb, |down| empty_waits += usize::from(down.is_empty())).unwrap();
            prop_assert_eq!(empty_waits, 0);

            let calls = &kb.calls;
            prop_assert_eq!(calls.last(), Some(&Call::Check(false)));
            prop_assert_eq!(calls[calls.len() - 2], Call::Grab);
            for (i, call) in calls.iter().enumerate() {
                match call {
                    Call::Grab => {
                        prop_assert_eq!(calls[i - 1], Call::Check(false));
                        if calls[i + 1] == Call::Check(true) {
                            prop_assert_eq!(calls[i + 2], Call::Ungrab);
                        }
                    }
                    Call::Ungrab => prop_assert_eq!(calls[i - 1], Call::Check(true)),
                    Call::Check(_) => {}
                }
            }
            let grabs = calls.iter().filter(|c| **c == Call::Grab).count();
            let ungrabs = calls.iter().filter(|c| **c == Call::Ungrab).count();
            prop_assert_eq!(grabs, ungrabs + 1);
        }
    }
}
