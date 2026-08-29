use niqol_core::{MarkService, WindowManager};
use niqol_niri::{NiriConnector, NiriEvent, NiriListener, NiriWindowManager};
use std::{env::var_os, sync::Arc};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tracing_subscriber::{EnvFilter, fmt};
use tracing::{debug};

mod actions;
mod daemon;
mod niri;
mod stores;

mod handlers;

use anyhow::Context;

use crate::handlers::EventHandler;


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

    let niri_wm: Arc<dyn WindowManager> =
        Arc::new(NiriWindowManager::connect(Arc::clone(&niri_connector)).await?);

    //mark service should need niri_wm
    let mark_service = Arc::new(MarkService::new(niri_wm));

    //mark service required in niri listener to listen to events
    //and update marks say if windows close (remove marks)
    let event_handler = EventHandler::new(mark_service);

    //mark service also required in action listener to handle events
    //such as marking windows and fetching window information and focusing marks

    let (niri_tx, mut niri_rx): (Sender<NiriEvent>, Receiver<NiriEvent>) = mpsc::channel(32);

    let niri_listener = NiriListener::new(niri_connector, niri_tx);

    tokio::try_join!(
        niri_listener.run(),
        async move {
            while let Some(niri_event) = niri_rx.recv().await {
                debug!("Handling niri event");
                event_handler
                    .handle_event(niri_event)
                    .await?;
            }
            Ok::<_, anyhow::Error>(())
        }
    )?;
    Ok(())
}
