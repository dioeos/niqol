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
    let weak_ui = ui.as_weak();

    tokio::spawn(async {
        match list_marks().await {
            Ok(QueryResponse::Marks(marks)) => {
                debug!("Loaded {} marks", marks.len());

                if let Err(err) = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = weak_ui.upgrade() {
                        let rows: Vec<MarkRowItem> = marks
                            .into_iter()
                            .map(|mark| MarkRowItem {
                                slot: mark.slot.to_string().into(),
                                title: mark
                                    .window
                                    .title
                                    .or(mark.window.app_id)
                                    .unwrap_or_else(|| "Untitled".to_owned())
                                    .into(),
                            })
                            .collect();
                        ui.set_marks(slint::ModelRc::new(slint::VecModel::from(rows)));
                        debug!("set marks");
                    }
                }) {
                    error!("failed to update marks UI: {err}");
                }
            }
            Err(err) => error!("failed to load marks: {err}"),
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
    Ok(query_response)
}
