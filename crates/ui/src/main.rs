use std::sync::Arc;

use niqol_core::{QueryRequest, QueryResponse};
use niqol_ipc::client::IpcClient;
use tokio::{runtime, sync::OnceCell};
use tracing::{debug, error, info};
use tracing_subscriber::{EnvFilter, fmt};

slint::include_modules!();

static IPC_CLIENT_CELL: OnceCell<Arc<IpcClient>> = OnceCell::const_new();

#[allow(unused_variables)]
fn main() -> Result<(), slint::PlatformError> {
    dotenvy::dotenv().ok();
    let format = fmt::format().with_level(true).with_target(true).compact();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .event_format(format)
        .init();
    let rt = runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_io()
        .build()
        .unwrap();

    let _rt_guard = rt.enter();
    info!("entered tokio rutime");

    let ui = AppWindow::new()?;
    let weak = ui.as_weak();

    tokio::spawn(async {
        match list_marks().await {
            Ok(QueryResponse::Marks(marks)) => debug!("Loaded {} marks", marks.len()),
            Err(err) => error!("Failed to load marks: {err}"),
        }
    });

    ui.run()
}

async fn use_ipc_client() -> Arc<IpcClient> {
    IPC_CLIENT_CELL
        .get_or_init(|| async { Arc::new(IpcClient::connect().await.unwrap()) })
        .await
        .clone()
}

async fn list_marks() -> Result<QueryResponse, niqol_ipc::error::Error> {
    let client = use_ipc_client().await;
    let query_response = client.request_query(QueryRequest::ListMarks).await?;
    debug!("Got marks");
    Ok(query_response)
}
