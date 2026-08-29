//! Types through the XDG RemoteDesktop portal (ADR 0004).

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, anyhow};
use ashpd::desktop::PersistMode;
use ashpd::desktop::Session;
use ashpd::desktop::remote_desktop::{DeviceType, KeyState, RemoteDesktop, SelectDevicesOptions};
use ashpd::enumflags2::BitFlags;
use ptw_core::typist::{Typist, TypistError};
use tracing::{info, warn};
use xkeysym::Keysym;

fn token_path() -> PathBuf {
    ptw_core::config::data_dir().join("portal-restore-token")
}

#[derive(Clone)]
pub struct PortalTypist {
    runtime: tokio::runtime::Handle,
    proxy: Arc<RemoteDesktop>,
    session: Arc<Session<RemoteDesktop>>,
}

impl PortalTypist {
    /// Opens a keyboard-only session. The first time, GNOME asks the user
    /// to allow remote control; the returned token skips the dialog after.
    pub async fn connect(runtime: tokio::runtime::Handle) -> anyhow::Result<Self> {
        let proxy = RemoteDesktop::new()
            .await
            .context("RemoteDesktop portal not available")?;
        let session = proxy.create_session(Default::default()).await?;
        let saved_token = std::fs::read_to_string(token_path())
            .ok()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty());
        proxy
            .select_devices(
                &session,
                SelectDevicesOptions::default()
                    .set_devices(BitFlags::from(DeviceType::Keyboard))
                    .set_persist_mode(PersistMode::ExplicitlyRevoked)
                    .set_restore_token(saved_token.as_deref()),
            )
            .await?
            .response()?;
        let devices = proxy
            .start(&session, None, Default::default())
            .await?
            .response()
            .context("remote control was not allowed")?;
        match devices.restore_token() {
            Some(token) => {
                let path = token_path();
                if let Some(dir) = path.parent() {
                    std::fs::create_dir_all(dir).ok();
                }
                if let Err(e) = std::fs::write(&path, token) {
                    warn!(error = %e, "cannot save portal restore token; the dialog will show again next start");
                }
            }
            None => warn!("portal gave no restore token; the dialog will show again next start"),
        }
        info!("portal keyboard session ready");
        Ok(Self {
            runtime,
            proxy: Arc::new(proxy),
            session: Arc::new(session),
        })
    }

    async fn type_async(&self, text: &str) -> ashpd::Result<()> {
        for ch in text.chars() {
            let keysym = keysym_for(ch);
            self.proxy
                .notify_keyboard_keysym(
                    &self.session,
                    keysym,
                    KeyState::Pressed,
                    Default::default(),
                )
                .await?;
            self.proxy
                .notify_keyboard_keysym(
                    &self.session,
                    keysym,
                    KeyState::Released,
                    Default::default(),
                )
                .await?;
        }
        Ok(())
    }
}

fn keysym_for(ch: char) -> i32 {
    let keysym = match ch {
        '\n' => Keysym::Return,
        '\t' => Keysym::Tab,
        _ => Keysym::from_char(ch),
    };
    keysym.raw() as i32
}

impl Typist for PortalTypist {
    fn type_text(&mut self, text: &str) -> Result<(), TypistError> {
        self.runtime
            .block_on(self.type_async(text))
            .map_err(|e| TypistError::Backend(e.to_string()))
    }
}

/// Whether the portal can be reached at all, for `ptw doctor`.
pub async fn probe() -> anyhow::Result<String> {
    let proxy = RemoteDesktop::new()
        .await
        .map_err(|e| anyhow!("RemoteDesktop portal: {e}"))?;
    let version = proxy.version();
    Ok(format!("RemoteDesktop portal v{version}"))
}
