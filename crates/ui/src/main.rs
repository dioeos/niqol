mod error;

use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

use niqol_core::Mark;
use niqol_ipc::client::IpcClient;
use slint::{Model, SharedString, VecModel};
use tokio::{runtime, sync::OnceCell};
use tracing::{debug, error, info};
use tracing_subscriber::{EnvFilter, fmt};

use crate::error::Error;

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
    resize_for_marks(&ui, 0);

    //@NOTE: Arc<Mutex<T>> is preferred over `Rc` due to `marks_state` being captured by a closure
    //       passed to `slint_invoke_from_event_loop` from a `tokio::spawn` task. `Rc` does not have
    //       `Send`, making it incorrect choice to wrap the state. The `all_marks_state` role is to
    //       share state between the tokio background worker thread and the main Slint event thread.
    let all_marks_state = Arc::new(Mutex::new(Vec::<MarkRowItem>::new()));
    let all_marks_state_for_initial_load = Arc::clone(&all_marks_state);

    let marks_model = Rc::new(VecModel::<MarkRowItem>::default());
    ui.set_marks(marks_model.into());

    let weak_ui = ui.as_weak();
    tokio::spawn(async move {
        let rows = match query_list_marks().await {
            Ok(rows) => rows,
            Err(err) => {
                error!("failed to query marks: {err}");
                return;
            }
        };

        *all_marks_state_for_initial_load.lock().unwrap() = rows.clone();

        weak_ui
            .upgrade_in_event_loop(move |ui| {
                let model = ui.get_marks();

                let model = model
                    .as_any()
                    .downcast_ref::<VecModel<MarkRowItem>>()
                    .expect("marks backed by Vec<T>");
                let marks_count = model.row_count();
                model.set_vec(rows);
                resize_for_marks(&ui, marks_count);
            })
            .unwrap()
    });

    let all_marks_state_for_filter_wrapper = Arc::clone(&all_marks_state);
    let weak_ui = ui.as_weak();
    ui.on_search_requested(move |search_input| {
        let query = search_input.to_string().to_lowercase();

        let filtered_rows = all_marks_state_for_filter_wrapper
            .lock()
            .unwrap()
            .iter()
            .filter(|row| row.title.to_lowercase().contains(&query))
            .cloned()
            .collect::<Vec<MarkRowItem>>();

        let Some(ui) = weak_ui.upgrade() else {
            return;
        };

        let model = ui.get_marks();
        let model = model
            .as_any()
            .downcast_ref::<VecModel<MarkRowItem>>()
            .expect("marks backed by Vec<T>");
        let count = filtered_rows.len();
        model.set_vec(filtered_rows);
        resize_for_marks(&ui, count);
    });
    ui.run()
}

fn resize_for_marks(ui: &AppWindow, count: usize) {
    ui.window()
        .set_size(slint::LogicalSize::new(500.0, 50.0 + count as f32 * 36.0));
}

async fn use_ipc_client() -> Arc<IpcClient> {
    IPC_CLIENT_CELL
        .get_or_init(|| async { Arc::new(IpcClient::connect().await.unwrap()) })
        .await
        .clone()
}

async fn query_list_marks() -> Result<Vec<MarkRowItem>, Error> {
    let client = use_ipc_client().await;
    let marks: Vec<Mark> = client.list_marks().await?;

    debug!(count = marks.len(), "queried marks");

    let rows: Vec<MarkRowItem> = marks
        .into_iter()
        .map(|mark| MarkRowItem {
            slot: SharedString::from(mark.slot.to_string()),
            title: SharedString::from(
                mark.window
                    .title
                    .or(mark.window.app_id)
                    .unwrap_or_else(|| "Untitled".into()),
            ),
        })
        .collect();
    Ok(rows)
}
