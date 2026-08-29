//! The tray icon: a microphone, light gray when idle and red while a
//! Dictation runs. Both states are drawn pixmaps: GNOME's AppIndicator
//! host prefers a theme icon name over a pixmap and keeps a stale pixmap
//! under a name, so mixing the two leaves ghosts.

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

    fn icon_pixmap(&self) -> Vec<Icon> {
        let color = if self.active { RED } else { LIGHT_GRAY };
        [16, 22, 24, 32, 48]
            .into_iter()
            .map(|size| microphone(size, color))
            .collect()
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

const RED: [u8; 3] = [0xe0, 0x1b, 0x24];
const LIGHT_GRAY: [u8; 3] = [0xde, 0xdd, 0xda];

/// A microphone glyph as ARGB32 (the StatusNotifierItem format): a
/// capsule, the cradle arc under it, a stem and a base.
fn microphone(size: i32, [r, g, b]: [u8; 3]) -> Icon {
    let scale = size as f32;
    let px = 1.0 / scale;
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let p = ((x as f32 + 0.5) / scale, (y as f32 + 0.5) / scale);
            let coverage = [
                capsule_distance(p),
                cradle_distance(p),
                rect_distance(p, (0.5, 0.775), (0.04, 0.065)),
                rect_distance(p, (0.5, 0.86), (0.19, 0.04)),
            ]
            .into_iter()
            .map(|d| (0.5 - d / px).clamp(0.0, 1.0))
            .fold(0.0, f32::max);
            data.extend_from_slice(&[(coverage * 255.0) as u8, r, g, b]);
        }
    }
    Icon {
        width: size,
        height: size,
        data,
    }
}

fn capsule_distance((x, y): (f32, f32)) -> f32 {
    let (cx, top, bottom, radius) = (0.5, 0.29, 0.43, 0.15);
    let yc = y.clamp(top, bottom);
    ((x - cx).powi(2) + (y - yc).powi(2)).sqrt() - radius
}

/// The lower half of a ring around the capsule.
fn cradle_distance((x, y): (f32, f32)) -> f32 {
    let (cx, cy, radius, thickness) = (0.5, 0.43, 0.25, 0.035);
    let ring = (((x - cx).powi(2) + (y - cy).powi(2)).sqrt() - radius).abs() - thickness;
    ring.max(cy - y)
}

fn rect_distance((x, y): (f32, f32), (cx, cy): (f32, f32), (hw, hh): (f32, f32)) -> f32 {
    let dx = (x - cx).abs() - hw;
    let dy = (y - cy).abs() - hh;
    (dx.max(0.0).powi(2) + dy.max(0.0).powi(2)).sqrt() + dx.max(dy).min(0.0)
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
