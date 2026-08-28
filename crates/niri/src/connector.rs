use std::path::PathBuf;

use anyhow::{Context, bail};
use niri_ipc::{Reply, Request, Response};
use tracing::{debug};
use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, net::UnixStream};


// both the action_listener and niri_listener need to connect
pub struct NiriConnector {
    niri_socket_path: PathBuf
}

impl NiriConnector {
    pub async fn new(niri_socket_path: PathBuf) -> Self {
        Self {
            niri_socket_path
        }
    }

    pub async fn connect(&self) -> anyhow::Result<UnixStream> {
        UnixStream::connect(&self.niri_socket_path)
            .await
            .with_context(|| format!(
                    "Failed to connect niri socket: {}",
                    self.niri_socket_path.display()
            ))
    }

    pub async fn send_event_stream_handshake(
        &self,
        reader: &mut BufReader<UnixStream>,
        buf: &mut String,
    ) -> anyhow::Result<()> {
        let stream = reader.get_mut();

        let mut payload: String = serde_json::to_string(&Request::EventStream)
            .context("Failed to serialize Request::EventStream to JSON")?;

        payload.push('\n');

        stream
            .write_all(payload.as_bytes())
            .await
            .context("Failed to send EventStream handshake to niri")?;

        stream
            .flush()
            .await
            .context("Failed to flush EventStream request to niri")?;

        buf.clear();

        let bytes_read = reader
            .read_line(buf)
            .await
            .context("Failed to read niri IPC reply from handshake")?;

        if bytes_read == 0 {
            bail!("niri closed the connection before acknowledging EventStream request");
        }

        let reply: Reply = serde_json::from_str(buf).context("Failed to parse niri EventStream handshake reply")?;

        match reply {
            Reply::Ok(Response::Handled) => {
                debug!("event stream handshake succeeded");
                Ok(())
            }
            _ => bail!("Handshake failed. Did not receive ack from niri")
        }
    }
}
