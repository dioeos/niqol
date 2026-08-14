use std::fs::File;
use std::io::Write;
use std::{path::PathBuf};
use anyhow::{Context, bail};
use tokio::net::UnixStream;
use tokio::io::{AsyncWriteExt, BufReader, AsyncBufReadExt};
use niri_ipc::{Reply, Request, Response};


pub struct NiriConnector {
    niri_socket_path: PathBuf
}

impl NiriConnector {
    pub fn new(niri_path: PathBuf) -> Self {
        Self {niri_socket_path: niri_path}
    }

    pub async fn connect(&self) -> Result<UnixStream, anyhow::Error> {
        UnixStream::connect(&self.niri_socket_path)
            .await
            .with_context(|| format!(
                    "Failed to connect to Niri socket: {}",
                    self.niri_socket_path.display()
            ))
    }

    pub async fn send_event_stream_handshake(
        &self,
        reader: &mut BufReader<UnixStream>,
        buf: &mut String,
        file: &mut File
    ) -> Result<(), anyhow::Error> {
        let stream = reader.get_mut();

        let mut payload: String = serde_json::to_string(&Request::EventStream)
            .context("Failed to serialize Request::EventStream to JSON")?;

        payload.push('\n');

        stream.write_all(payload.as_bytes())
            .await
            .context("Failed to send EventStream handshake to niri")?;

        stream.flush()
            .await
            .context("Failed to flush EventStream handshake to niri")?;

        buf.clear();

        let bytes_read = reader
            .read_line(buf)
            .await
            .context("Failed to read niri IPC reply from handshake")?;

        if bytes_read == 0 {
            bail!("Failed to read niri IPC reply from handshake");
        };

        let reply: Reply = serde_json::from_str(buf).context("Failed to parse niri handshake reply")?;

        match reply {
            Reply::Ok(Response::Handled) => {
                writeln!(file, "Handshake succeeded!")?;
            }
            _ => bail!("Handshake failed. Did not receive ack from niri")
        }
        Ok(())
    }
}
