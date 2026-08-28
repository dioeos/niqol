use niqol_niri::{NiriConnector, NiriEventHandler, NiriListener};
use std::{env::var_os, sync::Arc};
use tracing_subscriber::{EnvFilter, fmt};

mod actions;
mod daemon;
mod niri;
mod stores;

use anyhow::Context;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv().ok();
    let format = fmt::format().with_level(true).with_target(true).compact();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .event_format(format)
        .init();

    let niri_socket_path =
        var_os("NIRI_SOCKET").context("NIRI_SOCKET environment varialbe is not set")?;

    // let daemon = Daemon::new(niri_socket_path);
    // let niri_connector = NiriConnector::new(niri_socket_path.into());

    let niri_connector = Arc::new(NiriConnector::new(niri_socket_path.into()));

    let niri_event_handler = Arc::new(NiriEventHandler::new());

    let niri_listener = NiriListener::new(niri_connector, niri_event_handler);

    tokio::try_join!(niri_listener.run())?;
    Ok(())
}
