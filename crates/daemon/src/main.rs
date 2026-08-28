use std::{env::var_os, sync::Arc}; 
use niqol_niri::NiriConnector;
use tracing_subscriber::{fmt, EnvFilter};

mod daemon;
mod niri;
mod actions;
mod stores;

use anyhow::Context;
use daemon::Daemon;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv().ok();
    let format = fmt::format()
        .with_level(true)
        .with_target(true)
        .compact();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .event_format(format)
        .init();

    let niri_socket_path = var_os("NIRI_SOCKET")
        .context("NIRI_SOCKET environment varialbe is not set")?;

    // let daemon = Daemon::new(niri_socket_path);
    // let niri_connector = NiriConnector::new(niri_socket_path.into());

    let niri_connector = Arc::new(NiriConnector::new(niri_socket_path.into()));
    Ok(())
}
