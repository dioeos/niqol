use niqol_core::{ActionRequest, MarkService, QueryRequest, WindowManager, WindowService};
use niqol_niri::{NiriConnector, NiriEvent, NiriListener, NiriWindowManager};
use niqol_ipc::IpcSocket;
use std::{env::var_os, path::PathBuf, sync::Arc};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tracing::debug;
use tracing_subscriber::{EnvFilter, fmt};

mod handlers;
mod ipc_listener;

use anyhow::Context;

use crate::{
    handlers::{ActionHandler, EventHandler, QueryHandler},
};

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
        var_os("NIRI_SOCKET").context("NIRI_SOCKET environment variable is not set")?;

    // let daemon = Daemon::new(niri_socket_path);
    // let niri_connector = NiriConnector::new(niri_socket_path.into());

    let niri_connector = Arc::new(NiriConnector::new(niri_socket_path.into()));

    let niri_wm: Arc<dyn WindowManager> =
        Arc::new(NiriWindowManager::connect(Arc::clone(&niri_connector)).await?);

    //mark service should need niri_wm
    let mark_service = Arc::new(MarkService::new(niri_wm));
    let window_service = Arc::new(WindowService::new());

    //mark service required in niri listener to listen to events
    //and update marks say if windows close (remove marks)
    let event_handler = EventHandler::new(
        Arc::clone(&mark_service),
        Arc::clone(&window_service)
    );

    //mark service also required in action listener to handle events
    //such as marking windows and fetching window information and focusing marks

    let (niri_tx, mut niri_rx): (Sender<NiriEvent>, Receiver<NiriEvent>) = mpsc::channel(32);

    //niri listener is a dedicated EventStream connection
    let niri_listener = NiriListener::new(Arc::clone(&niri_connector), niri_tx);

    //create action socket
    let xdg_os_string =
        var_os("XDG_RUNTIME_DIR").context("XDG_RUNTIME_DIR environment variable is not set")?;

    let mut niqol_ipc_path = PathBuf::from(xdg_os_string);
    niqol_ipc_path.push("niqol-ipc.sock");

    let niqol_ipc_socket = IpcSocket::bind(niqol_ipc_path)?;

    let (action_tx, mut action_rx): (Sender<ActionRequest>, Receiver<ActionRequest>) =
        mpsc::channel(32);

    let (query_tx, mut query_rx): (Sender<QueryRequest>, Receiver<QueryRequest>) =
        mpsc::channel(32);


    let action_handler = ActionHandler::new(Arc::clone(&mark_service));
    let query_handler = QueryHandler::new(mark_service, window_service);

    let ipc_listener = ipc_listener::IpcListener::new(
        niqol_ipc_socket,
        action_tx,
        query_handler
    );

    tokio::try_join!(
        niri_listener.run(),   //listen to EventStream
        ipc_listener.run(),
        async move {
            while let Some(niri_event) = niri_rx.recv().await {
                debug!("Handling niri event");
                event_handler.handle_event(niri_event).await?;
            }
            Ok::<_, anyhow::Error>(())
        },
        async move {
            while let Some(action_request) = action_rx.recv().await {
                debug!("Handling action request");
                action_handler.handle_action_request(action_request).await?;
            }
            Ok::<_, anyhow::Error>(())
        },
    )?;
    Ok(())
}
