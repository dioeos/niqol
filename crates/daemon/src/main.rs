use niqol_core::{MarkService, WindowManager};
use niqol_niri::{NiriConnector, NiriEventHandler, NiriListener, NiriWindowManager};
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

    let niri_wm: Arc<dyn WindowManager> = Arc::new(
        NiriWindowManager::connect(Arc::clone(&niri_connector)).await?
    );

    //mark service should need niri_wm
    let mark_service = Arc::new(
        MarkService::new(niri_wm)
    );

    let niri_event_handler = Arc::new(NiriEventHandler::new());

    let niri_listener = NiriListener::new(niri_connector, niri_event_handler);

    //mark service required in niri listener to listen to events
    //and update marks say if windows close (remove marks)
    

    //mark service also required in action listener to handle events
    //such as marking windows and fetching window information and focusing marks

    tokio::try_join!(niri_listener.run())?;
    Ok(())
}
