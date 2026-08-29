# Desktop integration for ptw on Ubuntu 24.04 / GNOME 46 Wayland

Research date: 2026-08-28. Primary sources only (portal spec, docs.rs, GitLab/GitHub source at the gnome-46 branches, Ubuntu package index). Crate versions are what crates.io reported on that date.

## Target platform facts (Ubuntu 24.04 "noble")

| Component | Version | Source |
|---|---|---|
| GNOME Shell / mutter | 46.0 | packages.ubuntu.com/noble |
| Session | Wayland default; "Ubuntu on Xorg" still selectable at login | Ubuntu 24.04 release notes |
| xdg-desktop-portal | 1.18.4 | packages.ubuntu.com/noble |
| xdg-desktop-portal-gnome | 46.0 release, 46.2 in noble-updates; recommended by ubuntu-desktop | packages.ubuntu.com, launchpad.net/ubuntu/noble |
| gnome-shell-extension-appindicator | 57 (dependency of ubuntu-desktop) | packages.ubuntu.com/noble/ubuntu-desktop |
| GTK 4 / libadwaita | 4.14.2 / 1.5.0 | packages.ubuntu.com/noble |
| PipeWire (+ libpipewire-0.3-dev) | 1.0.5 | packages.ubuntu.com/noble |
| libei | 1.2.1 | packages.ubuntu.com/noble |
| systemd | 255 | packages.ubuntu.com/noble |
| ydotool in archive | 0.1.8 (pre-daemon syntax, old) | packages.ubuntu.com/noble |

For comparison, Ubuntu 26.04 ships GNOME 50, xdg-desktop-portal 1.21.1, xdg-desktop-portal-gnome 50, and drops the X11 session. Where a path needs that, it is marked "GNOME 48+" below.

Crate versions: ashpd 0.13.13, zbus 5.19.0, evdev 0.13.2, reis 0.7.1, enigo 0.6.1, ksni 0.3.6, tray-icon 0.24.2, global-hotkey 0.8.0, gtk4 0.11.4, libadwaita 0.9.2, relm4 0.11.0, iced 0.14.0, egui/eframe 0.36.1, winit 0.30.13, cpal 0.18.2, pipewire 0.10.1, rubato 5.0.0, arboard 3.6.1, wl-clipboard-rs 0.9.3, rdev 0.5.3 (last release 2023), device_query 4.0.1, handy-keys 0.3.4.

## 1. Global hotkey with press and release

### 1a. XDG GlobalShortcuts portal: not on GNOME 46

xdg-desktop-portal-gnome's NEWS: "48.rc: Add global shortcuts portal backend", "49.beta: Improvements to the Global Shortcuts portal", "50.beta: Properly send the Global Shortcut activation token to the portal frontend". Version 46.x has no GlobalShortcuts backend, and Ubuntu does not backport portal backends (noble-updates has 46.2). Calling `CreateSession` on 24.04 fails because no backend implements `org.freedesktop.impl.portal.GlobalShortcuts`. So on the target machine this path is off the table. It is still worth building behind a runtime check (`GlobalShortcuts::version()` errors when absent), because it is the clean answer on GNOME 48+ and KDE.

What it gives on GNOME 48+ (verified against current source, useful for when the box upgrades):

- Spec (interface v2): `Activated` and `Deactivated` signals with session, shortcut id, timestamp. The GNOME backend wires them to gnome-shell's `accelerator-activated` / `accelerator-deactivated`, and mutter's `handle_external_grab` calls `meta_display_accelerator_deactivate` on `CLUTTER_KEY_RELEASE`. So hold detection works and the compositor consumes the key.
- `preferred_trigger` uses the shortcuts spec string, `ALT+backslash`. gnome-control-center's dialog pre-fills it; the user can edit or cancel.
- The dialog appears on every `BindShortcuts` (by design, per GNOME maintainers). Call `ListShortcuts` first and only bind when the id is missing.
- xdg-desktop-portal 1.20+ rejects an empty app id for GlobalShortcuts. Host apps get an id from a systemd unit named `app-<id>.service` (regex in `xdp-app-info-host.c`) or by calling `org.freedesktop.host.portal.Registry.Register` first (`ashpd::register_host_app`). GNOME's provider also checks `g_application_id_is_valid`, so the id must be reverse-DNS, e.g. `dev.carver.ptw`.

### 1b. gnome-shell's own grab API: closed to us

`org.gnome.Shell.GrabAccelerator(s)` exists on GNOME 46 and emits `AcceleratorActivated`, but the gnome-46 `shellDBus.js` guards it with `new DBusSenderChecker(['org.gnome.Settings', 'org.gnome.SettingsDaemon.MediaKeys'])`, and there is no `AcceleratorDeactivated` signal on that branch (the deactivated plumbing was submitted for GNOME 47 along with the portal work). Extensions calling `Main.wm.addKeybinding` get press only: mutter 46 `keybindings.c` says "we used to have release-based bindings but no longer" and returns before handlers on `CLUTTER_KEY_RELEASE`; the gnome-46 `MetaKeyBindingFlags` enum has no release flag. Extensions also cannot see raw key events while a client window has focus on Wayland. So a shell extension can do toggle, not hold.

### 1c. evdev direct read (the only press+release path on GNOME 46 Wayland)

Mechanism with the `evdev` crate (0.13.2): `evdev::enumerate()`, keep devices whose `supported_keys()` contain `KEY_LEFTALT` and `KEY_BACKSLASH`, `into_event_stream()` (feature `tokio`), match `EventSummary::Key(_, key, value)` with value 1 press, 0 release, 2 autorepeat. Track modifier state yourself (Alt down, then backslash down = start; either up = stop). Watch `/dev/input` with inotify for hotplug. Works on every compositor, X11 and the console, so it doubles as the non-GNOME fallback.

Permissions: systemd's `50-udev-default.rules` sets `SUBSYSTEM=="input", GROUP="input"` (mode 0660), so `sudo usermod -aG input $USER` plus re-login is all that is needed to read. No udev rule. The cost is that any process running as the user can then read every keystroke; say so in setup output. This is what Handy's `handy-keys` crate does on Linux ("reads evdev devices (/dev/input/event*) directly, which works the same on Wayland, X11, and the console").

The leak. Reading does not consume the event, so Alt+\ still reaches the focused app. In bash/readline Alt+\ is `delete-horizontal-space`, so dictating into a terminal would eat spaces around the cursor every time. Options:

- `Device::grab()` (EVIOCGRAB, "prevents other clients ... from receiving events from this device") and re-emit everything else through a uinput virtual keyboard. That is a keyboard proxy like keyd; it needs `/dev/uinput` access and a very careful failure path (crash while grabbed = dead keyboard until the fd closes).
- Grab only while Alt is held and forward everything but backslash. Same machinery, narrower window, still needs uinput.
- Accept the leak and make the default key one that does nothing alone. Right Alt or Pause, as the COSMIC dictation write-up recommends. Cheapest; document that Alt+\ specifically misbehaves in terminals.

Crates compared: `rdev` 0.5.3 `listen` is X11 only ("will not work in Wayland"); its `grab` (feature `unstable_grab`) uses evdev and needs the `input` group; last release 2023 (Handy uses the rustdesk fork). `handy-keys` 0.3.4 is Handy's own evdev wrapper with `HotkeyState::Pressed/Released`; blocking needs `/dev/uinput`. `global-hotkey` 0.8.0 is "Linux (X11 Only)" but does report press and release, so it is the right tool if the user picks the Xorg session. `device_query` polls; skip. Use `evdev` directly.

### 1d. GNOME custom keybinding (gsettings), toggle only

Relocatable schema `org.gnome.settings-daemon.plugins.media-keys.custom-keybinding` with `name`, `command`, `binding` (`'<Alt>backslash'`), path appended to `org.gnome.settings-daemon.plugins.media-keys custom-keybindings`. gnome-settings-daemon grabs it and runs the command on press; no release. Good enough for a toggle mode (`ptw toggle` over D-Bus) with zero permissions and no key leak (the shell consumes the combo). This is the mode Handy's README tells Wayland users to set up. Offer it as the no-`input`-group option.

## 2. Typing text into the focused window on GNOME 46 Wayland

Constraints: mutter does not implement `zwp_virtual_keyboard_v1` (mutter issues #1974, #4124), so `wtype` and enigo's `wayland` feature fail with "Compositor does not support the virtual keyboard protocol". Mutter also implements neither wlr- nor ext-data-control (mutter #524), so a windowless daemon cannot set the clipboard through the usual tools.

### 2a. XDG RemoteDesktop portal (available on GNOME 46)

Verified for 24.04: xdg-desktop-portal 1.18.4's `remote-desktop.c` declares interface version 2, validates `persist_mode` and `restore_token` in `SelectDevices`, and has no app-type restriction on persistence (an empty host app id is fine, and 1.18 has no Registry to worry about). xdg-desktop-portal-gnome's gnome-46 `remotedesktop.c` implements `handle_connect_to_eis` and session restore (`serialize_session_as_restore_data` / `restore_from_data`); NEWS confirms both landed in 45.beta.

Flow with ashpd 0.13 (feature `remote_desktop`): `create_session` -> `select_devices(DeviceType::Keyboard, persist_mode = ExplicitlyRevoked, restore_token = saved)` -> `start` (GNOME shows a "remote control" dialog the first time; response carries a fresh `restore_token`, single-use, save it every time) -> then either:

- `notify_keyboard_keysym(session, keysym, KeyState)` over D-Bus. No libei. `XK_BackSpace` for deletions, `XK_Return` for newlines. Two calls per character.
- `connect_to_eis` and drive the fd with `reis` (ei client, `tokio` module, `Keyboard` interface) as enigo's libei backend does. Sends keycodes, so you resolve characters against the keymap the EIS server hands you. More code, fewer round trips.

Unicode limit on GNOME 46 (same code on gnome-46 and main): mutter's `meta-virtual-input-device-native.c` resolves keysyms against the current keymap only and drops the rest:

```c
if (!pick_keycode_for_keyval_in_current_group_in_impl (...))
  {
    g_warning ("No keycode found for keyval %x in current group", event->key);
    goto out;
  }
```

Anything typeable on the user's layout (including shifted level) works; emoji, curly quotes on a US layout, or "~" on a Swiss layout vanish with a warning. The fix upstream is libei 1.6's `ei_text` (utf8/keysym requests, May 2026) and mutter MR !4996, which was still a draft in March 2026 and is not in 46 or 50. On 24.04 (libei 1.2.1) it is off the table; substitute ASCII equivalents or use paste for those characters.

Works under XWayland too (mutter feeds the virtual device into the seat). Also works on KDE (with a layout bug, KDE #489021). xdg-desktop-portal-wlr does not implement RemoteDesktop, so Sway needs 2b.

### 2b. uinput virtual keyboard (universal fallback)

`evdev::uinput::VirtualDevice::builder()` declares keys, `emit(&[InputEvent])` appends SYN_REPORT. The compositor sees a real keyboard, so it works on GNOME, KDE, wlroots, X11, TTY. Keycodes only, so ptw must invert the user's xkb layout (xkbcommon crate; layout from `org.gnome.desktop.input-sources` or `XKB_DEFAULT_LAYOUT`). dotool does this and prints "impossible character for layout" for the rest. ydotool 0.1.8 (noble's version) and 1.0.4 both use a 128-entry `ascii2keycode_map` in `tool_type.c` and `type_char()` returns without emitting for unmapped bytes, so non-ASCII is silently dropped. Do not shell out to either; the evdev crate does the same in-process.

Permissions: `/dev/uinput` is root-only by default (systemd ships no rule for it). Install `/etc/udev/rules.d/99-ptw-uinput.rules` with `KERNEL=="uinput", MODE="0660", GROUP="input", OPTIONS+="static_node=uinput"`, then `udevadm control --reload && udevadm trigger`, plus the `input` group. Keep the virtual device open for the daemon's lifetime; a freshly created device takes a moment to be picked up (ydotool's README on why it needs a daemon).

### 2c. Clipboard paste (what Handy does)

Handy's `clipboard.rs`: set the clipboard through Tauri's clipboard plugin (it has a window, so Wayland lets it), then send the paste chord with the first tool found: KDE Wayland `kwtype`, other Wayland `wtype` -> `dotool` -> `ydotool`, X11 `xdotool` -> `ydotool`, else enigo; chord configurable (Ctrl+V, Ctrl+Shift+V, Shift+Insert); original clipboard restored afterwards. On GNOME Wayland `wtype` fails, so Handy users need dotool/ydotool installed, otherwise enigo's X11 backend through XWayland, which mutter 46.2+ routes through the RemoteDesktop portal as well (mutter #3507, "breaks xdotool key commands").

For a windowless ptw daemon, setting the clipboard on GNOME is the hard part (no data-control; arboard's Wayland path uses wl-clipboard-rs's data-control and fails; `wl-copy` maps an invisible focus-stealing surface). The portal `Clipboard` interface attached to a RemoteDesktop session is the clean way, and xdg-desktop-portal-gnome 46's restore data already carries a clipboard flag, but I did not verify the Clipboard portal end to end on 46. Paste is also wrong for streaming: Ctrl+V means different things in terminals, and you cannot revise pasted text without knowing exactly what the app inserted. Keep paste as an explicit "insert as block" mode for characters 2a cannot type.

### 2d. Revising typed text with backspaces

Every route can send BackSpace; the risk is on the receiving side:

- Autocorrect/autocomplete and IME composition change what a keystroke inserted (browser and editor completion especially).
- Editors auto-indent after Enter and auto-pair brackets and quotes; one BackSpace may delete two characters (VS Code deletes the pair) or none.
- Terminals: BackSpace works on the current line only.
- Dead keys and compose sequences are several events per character.
- Count grapheme clusters, not bytes or code points; most toolkits delete one cluster per BackSpace, some split ZWJ emoji.

Safe rules: revise only the unstable tail of the current utterance; never across a newline or punctuation already typed; cap the window (about 40 characters); commit words as soon as the recognizer marks them stable; expose a "type only final results" setting for apps where revision misbehaves. Handy sidesteps all of this by pasting the final transcript only.

## 3. System tray

`ksni` 0.3.6 implements StatusNotifierItem over D-Bus: implement `Tray` (`icon_name`, `icon_pixmap`, `menu`), run on tokio (default) or the `blocking` feature, keep the `Handle` and call `handle.update(|t| t.active = true)`; the trait methods are re-read on update, so a state-dependent icon is a match on a field. Ship the two states as embedded ARGB pixmaps so no icon theme install is needed. Menu: Toggle, Settings, Quit.

GNOME needs the AppIndicator extension for SNI. On 24.04, `ubuntu-desktop` depends on `gnome-shell-extension-appindicator` 57 and the Ubuntu session enables it, so the tray shows. Vanilla GNOME (Fedora, Debian GNOME) has no tray; ptw must work with no tray (CLI + hotkey) and treat the tray as a convenience. `tray-icon` 0.24.2 needs GTK3 plus libappindicator and a GTK main loop on the creating thread, wrong shape for a tokio daemon; skip it.

## 4. Settings dialog

The form is small: hotkey (recorder for evdev mode; on GNOME 48+ a button that calls the portal's `ConfigureShortcuts`), word list, microphone, model. The user prefers something lighter than GTK4/libadwaita, so the question is whether egui or iced is good enough on GNOME Wayland.

What "on Wayland" means for a winit app on GNOME: mutter offers no server-side decorations, so the toolkit must draw its own title bar. winit 0.30.13 has `wayland-csd-adwaita` (sctk-adwaita) on by default, but both egui-winit and eframe depend on winit with `default-features = false` and enable only `winit/wayland`, so a stock eframe window on GNOME Wayland has no title bar or close button. The fix is to add winit as a direct dependency with `features = ["wayland-csd-adwaita"]` (cargo unifies features), or draw a close button yourself. iced_winit's `wayland` feature enables `winit's wayland-csd-adwaita` already. Fractional scaling and HiDPI are handled by winit 0.30 on both.

- egui/eframe 0.36.1: immediate mode, quickest to write a four-field form; default renderer wgpu (Vulkan on Linux), `glow` feature for OpenGL if wgpu is a problem in a VM; eframe README apt deps `libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev`. Non-native look, but a settings dialog that opens from a tray does not need to match Adwaita. Add the winit CSD feature as above.
- iced 0.14.0: retained/Elm style, default features `wgpu`, `tiny-skia` (software fallback), `wayland`, `x11`, `linux-theme-detection`; CSD on by default. Slower iteration than egui for a tiny form, nicer widgets, larger dependency tree.
- gtk4-rs 0.11 + libadwaita 0.9: native look, dynamic links to the system GTK 4.14/libadwaita 1.5 (relm4's `gnome_46` feature matches), so a small binary, but `libgtk-4-dev libadwaita-1-dev` at build time and a GTK main loop that must live in its own process.
- Web UI: browser plus local HTTP server for four fields. No.

Any of these should run as a subprocess (`ptw settings`) talking to the daemon over D-Bus, so the daemon's tokio loop never hosts a GUI event loop and the GUI's dependencies never load in the daemon. Given that split and the user's preference, egui/eframe is good enough: the dialog is short-lived, opened from a tray menu, and the only Wayland-specific chore is the CSD feature flag. I did not measure binary sizes; expect eframe+wgpu to be the largest static binary and GTK the smallest, since GTK is dynamically linked.

## 5. Audio capture

cpal 0.18.2 added a native PipeWire host in 0.18.0 (June 2026, feature `pipewire`, needs `libpipewire-0.3-dev`, 1.0.5 on noble) and a PulseAudio host, and orders Linux hosts "PipeWire > PulseAudio > ALSA". cpal does not resample; you pick from `supported_input_configs()`. Whether cpal's PipeWire host lets you request 16 kHz mono and have PipeWire convert is not verified from its docs; PipeWire itself will ("running streams ... will automatically resample", client-side adaptive resampler).

pipewire-rs 0.10.1 (docs.rs failed to build the latest; 0.9.2 docs are fine): connect a capture `Stream` with `media.type=Audio, media.category=Capture, media.role=Communication`, offer one `EnumFormat` of F32 / 16000 / 1 channel; PipeWire resamples and downmixes; leave `target.object` unset and WirePlumber follows the default source; `node.latency = 320/16000` for 20 ms buffers. More boilerplate (main-loop thread, pod building), Linux-only, which the daemon is anyway.

Recommendation: cpal with `pipewire` feature (ALSA as fallback) plus `rubato` 5.0 to convert whatever rate you get to 16 kHz mono. Move to pipewire-rs only if cpal's two-month-old PipeWire host misbehaves on default-source changes, or if the mic-in-use indicator needs `media.role`.

## 6. On-screen indicator

- gtk4-layer-shell: "does not work on X11 or GNOME on Wayland"; mutter has no layer-shell. Handy links gtk-layer-shell and falls back to a plain Tauri window, which is why its overlay is off by default on Linux.
- Plain toolkit window: no always-on-top in GTK4 or winit on Wayland ("there isn't a programmatic way to manipulate the window stack, by design", E. Bassi), and in GNOME's default "smart" focus mode a newly mapped window takes focus, which would redirect the dictated text into the overlay. Strict mode shows an "is ready" notification instead. Unusable while dictating.
- Notification with `replaces_id`: `org.freedesktop.Notifications.Notify` must replace a live notification "atomically (ie with no flicker or other visual cues)"; a partial-transcript banner can be updated in place and closed on release, with no focus change. Downsides: it is the shell's banner UI, obeys Do Not Disturb, and may land in the notification list unless the `transient` hint is honoured (not verified on 46).
- GNOME Shell extension: the only way to draw a real overlay on GNOME; GNOME-only and a second codebase.

Recommendation: tray state, optional `replaces_id` notification for the partial transcript, extension later if wanted.

## 7. Daemon lifecycle

- systemd user unit `~/.config/systemd/user/app-dev.carver.ptw.service`: `Type=dbus`, `BusName=dev.carver.ptw`, `ExecStart=%h/.cargo/bin/ptw daemon`, `PartOf=graphical-session.target`, `After=graphical-session.target`, `WantedBy=graphical-session.target`. Ubuntu's GNOME session runs under systemd 255, so this is the right hook; an `~/.config/autostart/dev.carver.ptw.desktop` is only for non-systemd sessions. The `app-` prefix costs nothing now and hands the portal an app id on GNOME 48+.
- D-Bus activation `~/.local/share/dbus-1/services/dev.carver.ptw.service` with `SystemdService=app-dev.carver.ptw.service`, so `ptw toggle` starts the daemon on demand (this is what the gsettings toggle binding in 1d relies on).
- Single instance: zbus 5.19 `Connection::request_name("dev.carver.ptw")` "Fails with zbus::Error::NameTaken if the name is already owned by another peer"; on NameTaken the new process forwards its verb to the owner and exits.
- CLI: `ptw daemon | toggle | start | stop | status | settings | setup`. `setup` installs the udev rule (only if uinput mode is chosen), adds the user to `input`, writes the .desktop, unit and D-Bus service files, optionally the gsettings toggle binding, and prints what it did. D-Bus interface: `Toggle`, `Start`, `Stop`, `GetState`, `ReloadConfig`, signal `StateChanged` for tray and settings.

## Handy on Linux (the feel to match)

- Tauri v2, React + TypeScript front end, Rust back end.
- Hotkeys: two backends switchable at runtime in `shortcut/mod.rs`, `KeyboardImplementation::Tauri` (tauri-plugin-global-shortcut 2.3.1, X11 only on Linux) and `KeyboardImplementation::HandyKeys` (own `handy-keys` crate: evdev on Linux, `HotkeyState::Pressed/Released`), falling back to Tauri and persisting that if handy-keys cannot open `/dev/input` (no `input` group). Push-to-talk is `is_pressed` fed into a TranscriptionCoordinator. The "cancel" shortcut is compiled out on Linux "due to instability with dynamic shortcut registration". The README tells Wayland users to bind `handy --toggle-transcription` in their DE (issue #949: no shortcuts on COSMIC/Sway/Hyprland; a community Python evdev script calls the CLI on press and release).
- Typing: clipboard paste, tool chain in 2c, enigo 0.6.1 last. Discussion #718 proposes the RemoteDesktop portal for GNOME; today GNOME users install dotool or ydotool.
- Tray: Tauri's tray (tray-icon crate, libappindicator on Linux), three colored PNGs (idle/recording/transcribing) swapped with `set_icon`, a desired-state diff on the main thread; `--no-tray` flag.
- Overlay: gtk-layer-shell 0.8 when present (`KeyboardMode::None`, exclusive zone 0), else a non-focusable Tauri window; off by default on Linux because it steals focus.
- Audio: cpal 0.16, forked rodio, own resampler, Silero/earshot VAD.
- Transcription: transcribe-rs (onnx) and transcribe-cpp (whisper.cpp GGUF).

So Handy on GNOME Wayland is evdev hotkeys, clipboard plus uinput paste, appindicator tray, no overlay. ptw matches that on 24.04 and can be cleaner on typing (portal keysyms, no tools to install) and, after an upgrade to GNOME 48+, on hotkeys.

## Recommended stack for Ubuntu 24.04

| Concern | Choice on GNOME 46 | Later / elsewhere |
|---|---|---|
| Hotkey (hold) | evdev read, `input` group, default key that does nothing alone (right Alt or Pause); Alt+\ allowed with a terminal warning | GlobalShortcuts portal via ashpd on GNOME 48+ / KDE, behind a runtime check; global-hotkey on Xorg sessions |
| Hotkey (toggle, no permissions) | gsettings custom keybinding running `ptw toggle` | |
| Typing | RemoteDesktop portal, `NotifyKeyboardKeysym`, restore token saved after every start | uinput VirtualDevice + xkbcommon layout inversion (udev rule) on wlroots; paste mode for characters outside the layout |
| Revision | BackSpace over a capped unstable tail, never across newline; "final only" setting | |
| Tray | ksni, embedded pixmaps | none needed; daemon works without |
| Settings | egui/eframe in `ptw settings` subprocess with `winit/wayland-csd-adwaita`, D-Bus to daemon | iced if egui's look grates; gtk4+libadwaita if native look wins later |
| Audio | cpal 0.18 `pipewire` feature + rubato to 16 kHz mono | pipewire-rs direct |
| Indicator | tray state; optional Notifications `replaces_id` banner | GNOME extension |
| Lifecycle | `app-dev.carver.ptw.service` user unit, D-Bus activation, zbus name for single instance, CLI verbs | autostart .desktop |

Setup on 24.04 with this stack: `input` group membership for the hotkey (re-login), one remote-control dialog the first time typing runs, and nothing else. The udev rule only appears if the uinput typing fallback is enabled. Off the table on 24.04: GlobalShortcuts portal (needs xdg-desktop-portal-gnome 48), Registry host-app registration (needs xdg-desktop-portal 1.20), `ei_text` unicode injection (needs libei 1.6 and a mutter release that merged !4996), and anything layer-shell.

## Unverified

- Whether cpal 0.18's PipeWire host negotiates 16 kHz mono or only exposes the graph rate.
- Portal `Clipboard` interface working with a RemoteDesktop session on xdg-desktop-portal-gnome 46.
- Notification `transient` hint behaviour in GNOME Shell 46.
- Which mutter release, if any, merged MR !4996 (`ei_text`).
- Whether GNOME 48+'s `Activated` repeats on autorepeat and what happens if Alt is released before backslash (portal path, not relevant to 24.04).
- dotool's README (sr.ht returned 502); its layout behaviour is from its bug tracker.
- Binary sizes and cold-start times of eframe vs iced vs gtk4 builds.
- That the Ubuntu session on 24.04 enables the appindicator extension by default (package is a hard dependency; the enabled-extensions override was not fetched).
- Hyprland/COSMIC portal coverage for GlobalShortcuts and RemoteDesktop.

## Sources

- GlobalShortcuts portal spec: https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.GlobalShortcuts.html
- RemoteDesktop portal spec: https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.RemoteDesktop.html
- xdg-desktop-portal 1.18.4 remote-desktop.c: https://github.com/flatpak/xdg-desktop-portal/blob/1.18.4/src/remote-desktop.c
- Registry (host apps, 1.20+): https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.host.portal.Registry.html
- Host app id parsing: https://github.com/flatpak/xdg-desktop-portal/blob/main/shared/xdp-app-info-host.c
- RemoteDesktop persistence PR: https://github.com/flatpak/xdg-desktop-portal/pull/1004
- Shortcuts string spec: http://specifications.freedesktop.org/shortcuts/latest/
- xdg-desktop-portal-gnome NEWS: https://gitlab.gnome.org/GNOME/xdg-desktop-portal-gnome/-/raw/main/NEWS
- xdg-desktop-portal-gnome 46 remotedesktop.c: https://github.com/GNOME/xdg-desktop-portal-gnome/blob/gnome-46/src/remotedesktop.c
- xdg-desktop-portal-gnome globalshortcuts.c (48+): https://gitlab.gnome.org/GNOME/xdg-desktop-portal-gnome/-/raw/main/src/globalshortcuts.c
- gnome-control-center global-shortcuts-provider (48+): https://github.com/GNOME/gnome-control-center/tree/main/global-shortcuts-provider
- gnome-shell 46 shellDBus.js: https://github.com/GNOME/gnome-shell/blob/gnome-46/js/ui/shellDBus.js
- mutter 46 keybindings.c and prefs.h: https://github.com/GNOME/mutter/blob/gnome-46/src/core/keybindings.c, https://github.com/GNOME/mutter/blob/gnome-46/src/meta/prefs.h
- mutter 46 meta-virtual-input-device-native.c: https://github.com/GNOME/mutter/blob/gnome-46/src/backends/native/meta-virtual-input-device-native.c
- mutter main keybindings.c (handle_external_grab): https://github.com/GNOME/mutter/blob/main/src/core/keybindings.c
- mutter MR !4996 (ei_text): https://gitlab.gnome.org/GNOME/mutter/-/merge_requests/4996
- mutter virtual-keyboard-v1: https://gitlab.gnome.org/GNOME/mutter/-/issues/4124
- mutter data-control: https://gitlab.gnome.org/GNOME/mutter/-/work_items/524
- mutter xdotool/XTest via portal: https://gitlab.gnome.org/GNOME/mutter/-/issues/3507
- GNOME Discourse, global shortcuts popup: https://discourse.gnome.org/t/issue-with-global-shortcuts-setup-popup-in-gnome-48/27946
- GNOME Discourse, always-on-top: https://discourse.gnome.org/t/any-way-to-set-window-always-on-top-programmatically/31579
- GNOME Shell focus stealing prevention: https://blogs.gnome.org/shell-dev/2024/09/20/understanding-gnome-shells-focus-stealing-prevention/
- Portals with unsandboxed apps: https://blogs.gnome.org/ignapk/2025/06/04/using-portals-with-unsandboxed-apps/
- Electron GNOME 50 GlobalShortcuts issue: https://github.com/electron/electron/issues/51875
- libei text events: http://who-t.blogspot.com/2026/07/libei-and-keysymtext-events.html
- gnome-shell extensions and key events on Wayland: https://mail.gnome.org/archives/gnome-shell-extensions-list/2021-August/msg00000.html
- ashpd: https://docs.rs/ashpd/latest/ashpd/
- evdev: https://docs.rs/evdev/latest/evdev/
- reis: https://docs.rs/reis/latest/reis/
- enigo features and libei backend: https://docs.rs/crate/enigo/latest/features, https://github.com/enigo-rs/enigo/blob/main/src/linux/libei.rs
- rdev: https://github.com/Narsil/rdev
- handy-keys: https://docs.rs/handy-keys/latest/handy_keys/
- global-hotkey: https://docs.rs/global-hotkey/latest/global_hotkey/
- ksni: https://docs.rs/ksni/latest/ksni/
- tray-icon: https://docs.rs/tray-icon/latest/tray_icon/
- gnome-shell-extension-appindicator: https://github.com/ubuntu/gnome-shell-extension-appindicator
- gtk4-layer-shell: https://github.com/wmww/gtk4-layer-shell
- winit features: https://docs.rs/crate/winit/latest/features
- eframe and egui-winit Cargo.toml/features: https://docs.rs/crate/eframe/latest/source/Cargo.toml, https://docs.rs/crate/egui-winit/latest/source/Cargo.toml
- iced and iced_winit features: https://docs.rs/crate/iced/latest/features, https://docs.rs/crate/iced_winit/latest/features
- relm4: https://docs.rs/crate/relm4/latest
- cpal: https://docs.rs/cpal/latest/cpal/, https://github.com/RustAudio/cpal/blob/master/CHANGELOG.md, https://docs.rs/crate/cpal/latest/features
- PipeWire audio capture example and stream docs: https://docs.pipewire.org/audio-capture_8c-example.html, https://docs.pipewire.org/group__pw__stream.html
- zbus Connection: https://docs.rs/zbus/latest/zbus/connection/struct.Connection.html
- systemd udev defaults: https://github.com/systemd/systemd/blob/main/rules.d/50-udev-default.rules.in
- Notifications spec: http://specifications.freedesktop.org/notification/latest/protocol.html
- ydotool: https://github.com/ReimuNotMoe/ydotool (Client/tool_type.c)
- dotool tracker: https://todo.sr.ht/~geb/dotool/5
- wtype on GNOME: https://github.com/atx/wtype/issues/45
- Handy: https://github.com/cjpais/Handy (README, src-tauri/Cargo.toml, src/clipboard.rs, src/shortcut/*.rs, src/tray.rs, src/overlay.rs), issue #949, discussion #718
- COSMIC voice dictation write-up: https://codeshrew.github.io/ai-lab-notes/posts/2026-02-11_voice-dictation-cosmic-wayland/
- Ubuntu package index (noble): https://packages.ubuntu.com/noble/ ; Launchpad noble xdg-desktop-portal-gnome: https://launchpad.net/ubuntu/noble/+package/xdg-desktop-portal-gnome
- Ubuntu 26.04 release notes (for the forward-looking numbers): https://documentation.ubuntu.com/release-notes/26.04/summary-for-lts-users/
