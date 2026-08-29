---
status: accepted
---

# Type through the XDG RemoteDesktop portal, uinput as fallback

On GNOME 46 the Typist uses the RemoteDesktop portal (`NotifyKeyboardKeysym`): one "allow remote control" dialog ever, then a restore token, no extra setup, and it also reaches XWayland apps. The alternatives all fall short for a windowless streaming daemon: clipboard paste (Handy's route) cannot be revised or streamed and a windowless process cannot even set the clipboard on mutter; wtype/virtual-keyboard is refused by mutter; ydotool/uinput need a udev rule and silently drop anything outside ASCII. uinput stays as an opt-in fallback for compositors without the portal (wlroots).

## Consequences

- mutter drops keysyms not on the current keyboard layout, so characters outside the layout (curly quotes, emoji) need ASCII substitution. Recognizer output is plain text, so this rarely matters.
- Each character is two D-Bus calls. If that shows up in latency, switch the portal path to libei (`ConnectToEIS`), which batches.
