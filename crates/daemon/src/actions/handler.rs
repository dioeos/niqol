use std::{fs::File, sync::Arc, io::Write};

use anyhow::{Context, bail};
use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, net::UnixStream};
use tracing::{debug, info};

use crate::{actions::action::ActionRequest, stores::MarkStore};



pub struct ActionHandler {
    niri_reader: BufReader<UnixStream>,
    mark_store: Arc<MarkStore>
}

impl ActionHandler {
    pub fn new(
        niri_stream: UnixStream,
        mark_store: Arc<MarkStore>
    ) -> Self {
        Self {
            niri_reader: BufReader::new(niri_stream),
            mark_store
        }
    }

    pub async fn handle_action_request(
        &mut self,
        action_request: ActionRequest,
    ) -> Result<(), anyhow::Error> {
        let mut line = String::new();

        debug!("handling action request");

        match action_request {
            ActionRequest::MarkWindow { slot } => {
                let reply = self
                    .send_niri_request(&niri_ipc::Request::FocusedWindow, &mut line)
                    .await?
                    .map_err(anyhow::Error::msg)
                    .context("Niri reject FocusedWindow request")?;

                debug!(?reply, "received niri reply");

                let niri_window = match reply {
                    niri_ipc::Response::FocusedWindow(window) => window,
                    other => bail!("Expected FocusedWindow response, received: {other:?}")
                };

                let focused_window: niri_ipc::Window = niri_window
                    .context("Cannot mark window because no window is focused")?;

                self.mark_store.insert_mark(slot, focused_window.id).await;
            }
        }
        info!(?action_request, "action_completed");
        Ok(())
    }

    async fn send_niri_request(
        &mut self,
        request: &niri_ipc::Request,
        buf: &mut String
    ) -> Result<niri_ipc::Reply, anyhow::Error> {
        let stream = self.niri_reader.get_mut();

        let mut payload: String = serde_json::to_string(request)
            .context("Failed to serialize NiriRequest to JSON")?;

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

        let bytes_read = self.niri_reader
            .read_line(buf)
            .await
            .context("Failed to read niri IPC reply from niri request JSON payload")?;

        if bytes_read == 0 {
            bail!("Failed to read niri IPC reply from niri request JSON payload");
        };

        let reply: niri_ipc::Reply = serde_json::from_str(buf).context("Failed to parse niri request into reply")?;

        //can be of Reply::Err or Reply::Ok
        Ok(reply)

    }
}
