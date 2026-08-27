use std::sync::Arc;

use anyhow::{Context, bail};
use tracing::{debug, trace};
use niri_ipc::Event;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::UnixStream,
};
use tracing::info;

use crate::niri::NiriConnector;
use crate::stores::{MarkStore, WindowStore};

pub struct NiriListener {
    niri_connector: Arc<NiriConnector>,
    mark_store: Arc<MarkStore>,
    window_store: Arc<WindowStore>
}

impl NiriListener {
    pub fn new(
        connector: Arc<NiriConnector>,
        mark_store: Arc<MarkStore>,
        window_store: Arc<WindowStore>
    ) -> Self {
        Self {
            niri_connector: connector,
            mark_store,
            window_store
        }
    }

    #[tracing::instrument(
        name = "niri_listener",
        level = "info",
        skip(self)
    )]
    pub async fn run(&self) -> Result<(), anyhow::Error> {
        info!("listener started");
        let stream = self
            .niri_connector
            .connect()
            .await
            .context("Failed to initialize niri connection")?;

        let mut stream_reader: BufReader<UnixStream> = BufReader::new(stream);

        let mut line = String::new();

        self.niri_connector
            .send_event_stream_handshake(&mut stream_reader, &mut line)
            .await?;

        loop {
            let event: Event = Self::read_niri_stream(&mut stream_reader, &mut line).await?;

            Self::handle_niri_event(event).await?;
        }
    }

    async fn read_niri_stream(
        reader: &mut BufReader<UnixStream>,
        buf: &mut String,
    ) -> Result<niri_ipc::Event, anyhow::Error> {
        buf.clear();

        let bytes_read = reader
            .read_line(buf)
            .await
            .context("Failed to read niri IPC event")?;

        if bytes_read == 0 {
            bail!("niri IPC event stream closed unexpectedly");
        };

        let event: Event = serde_json::from_str(buf).context("Failed to parse niri IPC event")?;

        Ok(event)
    }

    async fn handle_niri_event(event: Event) -> Result<(), anyhow::Error> {
        match event {
            Event::WindowFocusChanged { id: Some(id) } => {
                debug!(window_id = id, "window focus changed");
            }
            Event::WorkspacesChanged { workspaces } => {
                debug!(workspace_count = workspaces.len(), "workspaces changed");
                trace!(workspaces = ?workspaces, "worksapce state");
            }
            _ => {
                trace!(?event, "ignoring niri event");
            }
        }
        Ok(())
    }
}
