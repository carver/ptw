//! The Key proxy: decides, key by key, what the compositor gets to see.
//!
//! With the keyboard grabbed (ADR 0006) nothing reaches the compositor
//! unless [`KeyProxy`] forwards it. The Hotkey chord is swallowed, so the
//! chord's modifier is never down for the apps while the Typist types.
//! A chord modifier press is deferred for a moment, because until the
//! next key arrives it could be the start of Alt+z or of Alt+Tab.

use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use crate::hotkey::{Chord, HotkeyEvent, HotkeyMachine, KeyAction, KeyEvent};
use crate::keys::{self, KeyCode};

/// How long a chord modifier press waits for the next key before the
/// compositor gets it, unless the config says otherwise. Chords land
/// 50-250 ms apart in practice; a chord slower than this leaks a bare Alt
/// tap, which Firefox and GTK apps take for a menu bar toggle. Alt+click
/// inside the window loses its Alt.
pub const DEFAULT_DEFERRAL: Duration = Duration::from_millis(500);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Output {
    /// Send to the compositor through the virtual keyboard.
    Forward(KeyEvent),
    Hotkey(HotkeyEvent),
    /// Whether the compositor now believes a modifier key is down.
    ModifiersHeld(bool),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// The keyboard is grabbed: only forwarded keys reach the compositor.
    Grab,
    /// No grab: the compositor sees every key, the proxy only reports.
    PassThrough,
}

#[derive(Clone, Debug)]
pub struct KeyProxy {
    mode: Mode,
    machine: HotkeyMachine,
    deferral: Duration,
    /// Deferred chord modifier presses, oldest first.
    deferred: Vec<(KeyCode, Instant)>,
    /// Keys the compositor believes are down.
    forwarded: BTreeSet<KeyCode>,
    modifiers_held: bool,
}

impl KeyProxy {
    pub fn new(chord: Chord, mode: Mode, deferral: Duration) -> Self {
        Self {
            mode,
            machine: HotkeyMachine::new(chord),
            deferral,
            deferred: Vec::new(),
            forwarded: BTreeSet::new(),
            modifiers_held: false,
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn chord(&self) -> &Chord {
        self.machine.chord()
    }

    /// Swaps the chord. Deferred presses go through, so no key is lost.
    pub fn set_chord(&mut self, chord: Chord) -> Vec<Output> {
        let mut out = Vec::new();
        self.forward_deferred(&mut out);
        self.machine = HotkeyMachine::new(chord);
        out
    }

    pub fn deferral(&self) -> Duration {
        self.deferral
    }

    /// Applies to presses already deferred as well.
    pub fn set_deferral(&mut self, deferral: Duration) {
        self.deferral = deferral;
    }

    /// When [`Self::on_deadline`] must next be called, if anything is deferred.
    pub fn deadline(&self) -> Option<Instant> {
        self.deferred.first().map(|(_, at)| *at + self.deferral)
    }

    pub fn on_deadline(&mut self, now: Instant) -> Vec<Output> {
        let mut out = Vec::new();
        self.expire(now, &mut out);
        self.report_modifiers(&mut out);
        out
    }

    pub fn on_key(&mut self, event: KeyEvent, now: Instant) -> Vec<Output> {
        let mut out = Vec::new();
        match self.mode {
            Mode::PassThrough => self.observe(event, &mut out),
            Mode::Grab => {
                self.expire(now, &mut out);
                self.proxy(event, now, &mut out);
            }
        }
        self.report_modifiers(&mut out);
        out
    }

    fn observe(&mut self, event: KeyEvent, out: &mut Vec<Output>) {
        if let Some(hotkey) = self.machine.on_key(event) {
            out.push(Output::Hotkey(hotkey));
        }
        match event.action {
            KeyAction::Press => {
                self.forwarded.insert(event.code);
            }
            KeyAction::Release => {
                self.forwarded.remove(&event.code);
            }
            KeyAction::Repeat => {}
        }
    }

    fn proxy(&mut self, event: KeyEvent, now: Instant, out: &mut Vec<Output>) {
        let code = event.code;
        let hotkey = self.machine.on_key(event);
        match event.action {
            KeyAction::Press => match hotkey {
                Some(HotkeyEvent::Start) => {
                    self.deferred.clear();
                    let chord_keys: Vec<KeyCode> = self
                        .forwarded
                        .iter()
                        .copied()
                        .filter(|c| self.machine.chord().is_member(*c))
                        .collect();
                    for chord_key in chord_keys {
                        self.forward(KeyEvent::release(chord_key), out);
                    }
                }
                Some(HotkeyEvent::Abort) | None if self.chord_key_to_swallow(code) => {}
                _ if self.machine.chord().is_member(code) && keys::is_modifier(code) => {
                    self.deferred.push((code, now));
                }
                _ => {
                    self.forward_deferred(out);
                    self.forward(event, out);
                }
            },
            KeyAction::Release => {
                if let Some(i) = self.deferred.iter().position(|(c, _)| *c == code) {
                    self.deferred.remove(i);
                    self.forward(KeyEvent::press(code), out);
                    self.forward(event, out);
                } else if self.forwarded.contains(&code) {
                    self.forward(event, out);
                }
            }
            KeyAction::Repeat => {
                if self.forwarded.contains(&code) {
                    out.push(Output::Forward(event));
                }
            }
        }
        if let Some(hotkey) = hotkey {
            out.push(Output::Hotkey(hotkey));
        }
    }

    /// A chord key pressed while the chord is held or latched belongs to
    /// the chord, not to the apps.
    fn chord_key_to_swallow(&self, code: KeyCode) -> bool {
        self.machine.chord().is_member(code) && !self.machine.is_idle()
    }

    fn expire(&mut self, now: Instant, out: &mut Vec<Output>) {
        while let Some((code, at)) = self.deferred.first().copied() {
            if now < at + self.deferral {
                break;
            }
            self.deferred.remove(0);
            self.forward(KeyEvent::press(code), out);
        }
    }

    fn forward_deferred(&mut self, out: &mut Vec<Output>) {
        for (code, _) in std::mem::take(&mut self.deferred) {
            self.forward(KeyEvent::press(code), out);
        }
    }

    fn forward(&mut self, event: KeyEvent, out: &mut Vec<Output>) {
        match event.action {
            KeyAction::Press => {
                self.forwarded.insert(event.code);
            }
            KeyAction::Release => {
                self.forwarded.remove(&event.code);
            }
            KeyAction::Repeat => {}
        }
        out.push(Output::Forward(event));
    }

    fn report_modifiers(&mut self, out: &mut Vec<Output>) {
        let held = self.forwarded.iter().any(|c| keys::is_modifier(*c));
        if held != self.modifiers_held {
            self.modifiers_held = held;
            out.push(Output::ModifiersHeld(held));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::{LEFT_ALT, LEFT_CTRL, LEFT_SHIFT, RIGHT_CTRL, key_code};

    fn proxy(chord: &str) -> KeyProxy {
        KeyProxy::new(chord.parse().unwrap(), Mode::Grab, DEFAULT_DEFERRAL)
    }

    fn fwd(event: KeyEvent) -> Output {
        Output::Forward(event)
    }

    fn z() -> KeyCode {
        key_code("z").unwrap()
    }

    fn tab() -> KeyCode {
        key_code("Tab").unwrap()
    }

    #[test]
    fn alt_z_is_swallowed_and_typing_is_clean() {
        let mut p = proxy("Alt+z");
        let t0 = Instant::now();
        assert_eq!(p.on_key(KeyEvent::press(LEFT_ALT), t0), vec![]);
        assert_eq!(p.deadline(), Some(t0 + DEFAULT_DEFERRAL));
        assert_eq!(
            p.on_key(KeyEvent::press(z()), t0 + Duration::from_millis(50)),
            vec![Output::Hotkey(HotkeyEvent::Start)]
        );
        assert_eq!(p.deadline(), None);
        assert_eq!(
            p.on_key(
                KeyEvent {
                    code: z(),
                    action: KeyAction::Repeat
                },
                t0 + Duration::from_millis(300)
            ),
            vec![]
        );
        assert_eq!(
            p.on_key(KeyEvent::release(z()), t0 + Duration::from_secs(3)),
            vec![Output::Hotkey(HotkeyEvent::Stop)]
        );
        assert_eq!(
            p.on_key(KeyEvent::release(LEFT_ALT), t0 + Duration::from_secs(4)),
            vec![]
        );
    }

    #[test]
    fn alt_tab_goes_through_when_tab_arrives() {
        let mut p = proxy("Alt+z");
        let t0 = Instant::now();
        assert_eq!(p.on_key(KeyEvent::press(LEFT_ALT), t0), vec![]);
        assert_eq!(
            p.on_key(KeyEvent::press(tab()), t0 + Duration::from_millis(50)),
            vec![
                fwd(KeyEvent::press(LEFT_ALT)),
                fwd(KeyEvent::press(tab())),
                Output::ModifiersHeld(true)
            ]
        );
        assert_eq!(
            p.on_key(KeyEvent::release(tab()), t0 + Duration::from_millis(100)),
            vec![fwd(KeyEvent::release(tab()))]
        );
        assert_eq!(
            p.on_key(KeyEvent::release(LEFT_ALT), t0 + Duration::from_millis(150)),
            vec![
                fwd(KeyEvent::release(LEFT_ALT)),
                Output::ModifiersHeld(false)
            ]
        );
    }

    /// Seen on the host 2026-09-09: chords pressed 225 ms apart leaked an
    /// Alt tap past a 150 ms Deferral, and Firefox opened its menu bar.
    #[test]
    fn a_chord_pressed_a_quarter_second_apart_is_still_swallowed() {
        let mut p = proxy("Alt+z");
        let t0 = Instant::now();
        p.on_key(KeyEvent::press(LEFT_ALT), t0);
        assert_eq!(p.on_deadline(t0 + Duration::from_millis(250)), vec![]);
        assert_eq!(
            p.on_key(KeyEvent::press(z()), t0 + Duration::from_millis(250)),
            vec![Output::Hotkey(HotkeyEvent::Start)]
        );
    }

    #[test]
    fn a_slow_alt_is_forwarded_at_the_deadline() {
        let mut p = proxy("Alt+z");
        let t0 = Instant::now();
        p.on_key(KeyEvent::press(LEFT_ALT), t0);
        assert_eq!(p.on_deadline(t0 + Duration::from_millis(100)), vec![]);
        assert_eq!(
            p.on_deadline(t0 + DEFAULT_DEFERRAL),
            vec![fwd(KeyEvent::press(LEFT_ALT)), Output::ModifiersHeld(true)]
        );
        assert_eq!(p.deadline(), None);
        // Tab now goes straight through: Alt+Tab still works.
        assert_eq!(
            p.on_key(KeyEvent::press(tab()), t0 + Duration::from_millis(200)),
            vec![fwd(KeyEvent::press(tab()))]
        );
    }

    #[test]
    fn a_forwarded_alt_is_released_when_the_chord_completes() {
        let mut p = proxy("Alt+z");
        let t0 = Instant::now();
        p.on_key(KeyEvent::press(LEFT_ALT), t0);
        p.on_deadline(t0 + DEFAULT_DEFERRAL);
        assert_eq!(
            p.on_key(KeyEvent::press(z()), t0 + Duration::from_millis(300)),
            vec![
                fwd(KeyEvent::release(LEFT_ALT)),
                Output::Hotkey(HotkeyEvent::Start),
                Output::ModifiersHeld(false)
            ]
        );
        assert_eq!(
            p.on_key(KeyEvent::release(z()), t0 + Duration::from_secs(2)),
            vec![Output::Hotkey(HotkeyEvent::Stop)]
        );
        // The physical Alt release must not reach the compositor twice.
        assert_eq!(
            p.on_key(KeyEvent::release(LEFT_ALT), t0 + Duration::from_secs(3)),
            vec![]
        );
    }

    #[test]
    fn an_alt_tap_is_still_a_tap() {
        let mut p = proxy("Alt+z");
        let t0 = Instant::now();
        p.on_key(KeyEvent::press(LEFT_ALT), t0);
        assert_eq!(
            p.on_key(KeyEvent::release(LEFT_ALT), t0 + Duration::from_millis(80)),
            vec![
                fwd(KeyEvent::press(LEFT_ALT)),
                fwd(KeyEvent::release(LEFT_ALT))
            ]
        );
        assert_eq!(p.deadline(), None);
    }

    #[test]
    fn other_keys_pass_untouched_and_a_chord_letter_is_not_deferred() {
        let mut p = proxy("Alt+z");
        let t0 = Instant::now();
        let x = key_code("x").unwrap();
        assert_eq!(
            p.on_key(KeyEvent::press(x), t0),
            vec![fwd(KeyEvent::press(x))]
        );
        assert_eq!(
            p.on_key(KeyEvent::release(x), t0),
            vec![fwd(KeyEvent::release(x))]
        );
        assert_eq!(
            p.on_key(KeyEvent::press(z()), t0),
            vec![fwd(KeyEvent::press(z()))]
        );
        assert_eq!(
            p.on_key(KeyEvent::release(z()), t0),
            vec![fwd(KeyEvent::release(z()))]
        );
    }

    #[test]
    fn another_key_during_the_hold_aborts_and_goes_through_alone() {
        let mut p = proxy("Alt+z");
        let t0 = Instant::now();
        p.on_key(KeyEvent::press(LEFT_ALT), t0);
        p.on_key(KeyEvent::press(z()), t0);
        assert_eq!(
            p.on_key(KeyEvent::press(tab()), t0),
            vec![
                fwd(KeyEvent::press(tab())),
                Output::Hotkey(HotkeyEvent::Abort)
            ]
        );
        assert_eq!(
            p.on_key(KeyEvent::release(tab()), t0),
            vec![fwd(KeyEvent::release(tab()))]
        );
        assert_eq!(p.on_key(KeyEvent::release(z()), t0), vec![]);
        assert_eq!(p.on_key(KeyEvent::release(LEFT_ALT), t0), vec![]);
    }

    #[test]
    fn a_lone_modifier_chord_never_reaches_the_compositor() {
        let mut p = proxy("RightCtrl");
        let t0 = Instant::now();
        assert_eq!(
            p.on_key(KeyEvent::press(RIGHT_CTRL), t0),
            vec![Output::Hotkey(HotkeyEvent::Start)]
        );
        assert_eq!(
            p.on_key(KeyEvent::release(RIGHT_CTRL), t0),
            vec![Output::Hotkey(HotkeyEvent::Stop)]
        );
        assert_eq!(
            p.on_key(KeyEvent::press(LEFT_CTRL), t0),
            vec![fwd(KeyEvent::press(LEFT_CTRL)), Output::ModifiersHeld(true)]
        );
    }

    #[test]
    fn two_deferred_modifiers_go_through_in_order() {
        let mut p = proxy("Ctrl+Shift+Space");
        let t0 = Instant::now();
        let a = key_code("a").unwrap();
        p.on_key(KeyEvent::press(LEFT_CTRL), t0);
        p.on_key(KeyEvent::press(LEFT_SHIFT), t0 + Duration::from_millis(20));
        assert_eq!(
            p.on_key(KeyEvent::press(a), t0 + Duration::from_millis(40)),
            vec![
                fwd(KeyEvent::press(LEFT_CTRL)),
                fwd(KeyEvent::press(LEFT_SHIFT)),
                fwd(KeyEvent::press(a)),
                Output::ModifiersHeld(true)
            ]
        );
    }

    #[test]
    fn a_non_chord_modifier_held_during_the_hold_blocks_typing() {
        let mut p = proxy("Alt+z");
        let t0 = Instant::now();
        assert_eq!(
            p.on_key(KeyEvent::press(LEFT_SHIFT), t0),
            vec![
                fwd(KeyEvent::press(LEFT_SHIFT)),
                Output::ModifiersHeld(true)
            ]
        );
        p.on_key(KeyEvent::press(LEFT_ALT), t0);
        assert_eq!(
            p.on_key(KeyEvent::press(z()), t0),
            vec![Output::Hotkey(HotkeyEvent::Start)]
        );
        assert_eq!(
            p.on_key(KeyEvent::release(LEFT_SHIFT), t0),
            vec![
                fwd(KeyEvent::release(LEFT_SHIFT)),
                Output::ModifiersHeld(false)
            ]
        );
    }

    #[test]
    fn changing_the_chord_releases_deferred_presses() {
        let mut p = proxy("Alt+z");
        let t0 = Instant::now();
        p.on_key(KeyEvent::press(LEFT_ALT), t0);
        assert_eq!(
            p.set_chord("RightCtrl".parse().unwrap()),
            vec![fwd(KeyEvent::press(LEFT_ALT))]
        );
        assert_eq!(p.chord().to_string(), "RightCtrl");
    }

    #[test]
    fn pass_through_only_reports() {
        let mut p = KeyProxy::new(
            "Alt+z".parse().unwrap(),
            Mode::PassThrough,
            DEFAULT_DEFERRAL,
        );
        let t0 = Instant::now();
        assert_eq!(
            p.on_key(KeyEvent::press(LEFT_ALT), t0),
            vec![Output::ModifiersHeld(true)]
        );
        assert_eq!(
            p.on_key(KeyEvent::press(z()), t0),
            vec![Output::Hotkey(HotkeyEvent::Start)]
        );
        assert_eq!(
            p.on_key(KeyEvent::release(z()), t0),
            vec![Output::Hotkey(HotkeyEvent::Stop)]
        );
        assert_eq!(
            p.on_key(KeyEvent::release(LEFT_ALT), t0),
            vec![Output::ModifiersHeld(false)]
        );
        assert_eq!(p.deadline(), None);
    }
}
