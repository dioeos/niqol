#![allow(dead_code)]
use std::sync::Arc;

use niqol_core::{MarkService, WindowId, WindowService};
use niqol_niri::{NiriEvent, from_niri_window};
use tracing::{debug, trace};

pub(crate) struct EventHandler {
    mark_service: Arc<MarkService>,
    window_service: Arc<WindowService>,
}

impl EventHandler {
    pub(crate) fn new(mark_service: Arc<MarkService>, window_service: Arc<WindowService>) -> Self {
        Self {
            mark_service,
            window_service,
        }
    }

    pub(crate) async fn handle_event(&self, event: NiriEvent) -> anyhow::Result<()> {
        match event {
            NiriEvent::WindowFocusChanged { id: Some(id) } => {
                debug!(window_id = id, "window focus changed");
            }
            NiriEvent::WindowOpenedOrChanged { window } => {
                debug!(window = ?window, "window opened or changed");
                let niqol_window = from_niri_window(window);
                self.window_service.insert_window(niqol_window).await;
            }
            NiriEvent::WindowClosed { id } => {
                debug!(window_id = id, "window closed");
                self.window_service.remove_window(WindowId(id)).await;
            }
            _ => {
                trace!(?event, "ignoring niri event");
            }
        }
        Ok(())
    }
}
