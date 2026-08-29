//! The tray icon: the theme's microphone when idle, a solid red disc
//! while a Dictation runs. Hosts prefer an icon name over a pixmap, so
//! the active state gives no name at all.

use std::sync::mpsc::Sender;

use ksni::menu::{MenuItem, StandardItem};
use ksni::{Handle, Icon, Tray, TrayMethods};

use crate::daemon::Command;

pub struct PtwTray {
    active: bool,
    commands: Sender<Command>,
}

impl Tray for PtwTray {
    fn id(&self) -> String {
        "ptw".into()
    }

    fn title(&self) -> String {
        "Push to Whisper".into()
    }

    fn icon_name(&self) -> String {
        if self.active {
            String::new()
        } else {
            "audio-input-microphone-symbolic".into()
        }
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        if self.active {
            vec![
                red_disc(16),
                red_disc(22),
                red_disc(24),
                red_disc(32),
                red_disc(48),
            ]
        } else {
            Vec::new()
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        self.commands.send(Command::OpenSettings).ok();
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        vec![
            StandardItem {
                label: if self.active {
                    "Transcribing...".into()
                } else {
                    "Idle".into()
                },
                enabled: false,
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Settings...".into(),
                activate: Box::new(|tray: &mut Self| {
                    tray.commands.send(Command::OpenSettings).ok();
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Quit".into(),
                activate: Box::new(|tray: &mut Self| {
                    tray.commands.send(Command::Quit).ok();
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

/// ARGB32, as the StatusNotifierItem spec wants.
fn red_disc(size: i32) -> Icon {
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    let center = (size as f32 - 1.0) / 2.0;
    let radius = size as f32 * 0.46;
    for y in 0..size {
        for x in 0..size {
            let d = ((x as f32 - center).powi(2) + (y as f32 - center).powi(2)).sqrt();
            let alpha = (radius + 0.5 - d).clamp(0.0, 1.0);
            data.extend_from_slice(&[(alpha * 255.0) as u8, 0xe0, 0x1b, 0x24]);
        }
    }
    Icon {
        width: size,
        height: size,
        data,
    }
}

pub async fn spawn(commands: Sender<Command>) -> Result<Handle<PtwTray>, ksni::Error> {
    PtwTray {
        active: false,
        commands,
    }
    .spawn()
    .await
}

pub async fn set_active(handle: &Handle<PtwTray>, active: bool) {
    handle.update(|tray| tray.active = active).await;
}
