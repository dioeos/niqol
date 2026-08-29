use std::sync::Arc;

use anyhow::{Context, bail};
use async_trait::async_trait;
use niqol_core::{Window, WindowId, WindowManager};
use niri_ipc::{Action, Reply, Request, Response};
use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, net::UnixStream, sync::Mutex};
use tracing::debug;

use crate::{NiriConnector, conversions::from_niri_window};

pub struct NiriWindowManager {
    stream_reader_connection: Mutex<BufReader<UnixStream>>
}

impl NiriWindowManager {
    pub async fn connect(
        connector: Arc<NiriConnector>
    ) -> anyhow::Result<Self> {
        let stream = connector
            .connect()
            .await
            .context("Failed to initialize niri command connection")?;

        Ok(Self::from_stream(stream))
    }

    fn from_stream(
        stream: UnixStream
    ) -> Self {
        Self { stream_reader_connection: Mutex::new(BufReader::new(stream)) }
    }

    async fn send_niri_request(
        &self,
        request: &Request,
        buf: &mut String
    ) -> anyhow::Result<Reply> {
        let mut payload: String = serde_json::to_string(&request)
            .context("Failed to serialize niri request to JSON")?;

        let mut connection_guard = self
            .stream_reader_connection
            .lock()
            .await;

        let stream: &mut UnixStream = connection_guard.get_mut();

        payload.push('\n');

        stream
            .write_all(payload.as_bytes())
            .await
            .context("Failed to send niri request JSON payload")?;

        stream
            .flush()
            .await
            .context("Failed to flush niri request JSON payload")?;

        buf.clear();

        let bytes_read = connection_guard
            .read_line(buf)
            .await
            .context("Failed to read niri IPC reply from niri request JSON payload")?;

        if bytes_read == 0 {
            bail!("niri closed the connection before acknowledging niri request");
        }

        let reply: Reply = serde_json::from_str(buf).context("Failed to deserialize niri request into reply")?;

        //can be of Reply::Err or Reply::Ok(Response::<T>)
        Ok(reply)
    }
}

#[async_trait]
impl WindowManager for NiriWindowManager {
    async fn get_focused_window(
        &self
    ) -> anyhow::Result<Option<Window>> {
        let mut buf = String::new();

        let response = self
            .send_niri_request(&Request::FocusedWindow, &mut buf)
            .await?
            .map_err(anyhow::Error::msg)
            .context("niri rejected FocusedWindow request")?;

        let niri_window = match response {
            Response::FocusedWindow(Some(window)) => Some(window),
            Response::FocusedWindow(None) => {
                debug!("No focused window");
                None
            }
            other => bail!("Expected FocusedWindow response, received: {other:?}")
        };

        let domain_window = niri_window.map(from_niri_window);

        Ok(domain_window)
    }

    async fn focus_window(
        &self,
        id: WindowId
    ) -> anyhow::Result<()> {
        let mut buf = String::new();

        let action = Action::FocusWindow { id: id.0 };
        let request = Request::Action(action);

        let response = self
            .send_niri_request(&request, &mut buf)
            .await?
            .map_err(anyhow::Error::msg)
            .context("niri rejected FocusWindow action request")?;

        match response {
            Response::Handled => {
                Ok(())
            }
            other => bail!("Expected Handled response, received: {other:?}")
        }
    }
}
