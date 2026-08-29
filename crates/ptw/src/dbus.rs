//! The daemon's D-Bus name doubles as its single-instance lock; `ptw toggle`
//! is a method call on it.

use std::sync::mpsc::Sender;

use anyhow::Context;

use crate::daemon::Command;

pub const BUS_NAME: &str = "dev.carver.ptw";
pub const OBJECT_PATH: &str = "/dev/carver/ptw";

pub struct DaemonInterface {
    commands: Sender<Command>,
}

#[zbus::interface(name = "dev.carver.ptw.Daemon")]
impl DaemonInterface {
    /// Start a Dictation if idle, stop it if one is running.
    fn toggle(&self) {
        self.commands.send(Command::Toggle).ok();
    }

    fn open_settings(&self) {
        self.commands.send(Command::OpenSettings).ok();
    }

    fn quit(&self) {
        self.commands.send(Command::Quit).ok();
    }
}

/// Claims the bus name. Fails when another daemon holds it.
pub async fn serve(commands: Sender<Command>) -> anyhow::Result<zbus::Connection> {
    zbus::connection::Builder::session()?
        .name(BUS_NAME)?
        .serve_at(OBJECT_PATH, DaemonInterface { commands })?
        .build()
        .await
        .context("another ptw daemon is already running (or D-Bus is unavailable)")
}

#[zbus::proxy(
    interface = "dev.carver.ptw.Daemon",
    default_service = "dev.carver.ptw",
    default_path = "/dev/carver/ptw"
)]
trait Daemon {
    fn toggle(&self) -> zbus::Result<()>;
    fn open_settings(&self) -> zbus::Result<()>;
    fn quit(&self) -> zbus::Result<()>;
}

fn client() -> anyhow::Result<DaemonProxyBlocking<'static>> {
    let connection = zbus::blocking::Connection::session()?;
    DaemonProxyBlocking::new(&connection).context("connect to ptw daemon")
}

pub fn toggle() -> anyhow::Result<()> {
    client()?
        .toggle()
        .context("is the ptw daemon running? start it with `ptw daemon`")
}

/// True when a daemon answers on the bus.
pub fn daemon_running() -> bool {
    let Ok(connection) = zbus::blocking::Connection::session() else {
        return false;
    };
    let Ok(proxy) = zbus::blocking::fdo::DBusProxy::new(&connection) else {
        return false;
    };
    proxy
        .name_has_owner(BUS_NAME.try_into().expect("valid bus name"))
        .unwrap_or(false)
}
