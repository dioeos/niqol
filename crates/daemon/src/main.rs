use std::{env::var_os}; 

mod daemon;
mod niri;

use anyhow::Context;
use daemon::Daemon;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let niri_socket_path = var_os("NIRI_SOCKET")
        .context("NIRI_SOCKET environment varialbe is not set")?;

    let daemon = Daemon::new(niri_socket_path);

    // daemon.heartbeat()?;
    daemon.run().await?;
    Ok(())
}
