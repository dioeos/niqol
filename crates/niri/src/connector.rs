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
    pub fn new(niri_socket_path: PathBuf) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::{
        io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
        net::UnixListener
    };

    fn socket_path() -> (tempfile::TempDir, PathBuf) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("niri-test.sock");
        (dir, path)
    }

    #[tokio::test]
    async fn connect_succeeds_when_socket_exists() {
        let (_dir, path) = socket_path();
        let listener = UnixListener::bind(&path).unwrap();

        let accept_task = tokio::spawn(async move {
            let (_stream, _) = listener.accept().await.unwrap();
        });

        let connector = NiriConnector::new(path);

        connector.connect().await.unwrap();

        accept_task.await.unwrap();
    }

    #[tokio::test]
    async fn connect_fails_when_socket_missing() {
        let (_dir, path) = socket_path();

        let connector = NiriConnector::new(path);

        let err = connector.connect().await.unwrap_err();

        assert!(
            err.to_string().contains("Failed to connect niri socket")
        );
    }

    #[tokio::test]
    async fn send_event_stream_handshake_sends_event_stream_request() {
        let (_dir, path) = socket_path();
        let listener = UnixListener::bind(&path).unwrap();

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut reader = BufReader::new(stream);

            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();

            let request: Request = serde_json::from_str(&line).unwrap();

            assert!(matches!(request, Request::EventStream));

            let reply = Reply::Ok(Response::Handled);
            let mut response = serde_json::to_string(&reply).unwrap();
            response.push('\n');

            reader.get_mut().write_all(response.as_bytes()).await.unwrap();
            reader.get_mut().flush().await.unwrap();
        });

        let connector = NiriConnector::new(path);
        let stream = connector.connect().await.unwrap();
        let mut reader = BufReader::new(stream);
        let mut buf = String::new();

        connector
            .send_event_stream_handshake(&mut reader, &mut buf)
            .await
            .unwrap();

        server.await.unwrap();
    }

    #[tokio::test]
    async fn send_event_stream_handshake_fails_when_server_disconnects() {
        let (_dir, path) = socket_path();
        let listener = UnixListener::bind(&path).unwrap();

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut reader = BufReader::new(stream);

            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();

            //connection drops (no reply)
        });

        let connector = NiriConnector::new(path);
        let stream = connector.connect().await.unwrap();
        let mut reader = BufReader::new(stream);
        let mut buf = String::new();

        let err = connector
            .send_event_stream_handshake(&mut reader, &mut buf)
            .await
            .unwrap_err();

        assert!(
            err.to_string()
                .contains("niri closed the connection before acknowledging EventStream")
        );

        server.await.unwrap();
    }

    #[tokio::test]
    async fn send_event_stream_handshake_fails_on_unexpected_reply() {
        let (_dir, path) = socket_path();
        let listener = UnixListener::bind(&path).unwrap();

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut reader = BufReader::new(stream);
            let mut line = String::new();

            reader.read_line(&mut line).await.unwrap();
            let reply = Reply::Err("request rejected".into());

            let mut response = serde_json::to_string(&reply).unwrap();
            response.push('\n');

            reader.get_mut().write_all(response.as_bytes()).await.unwrap();
            reader.get_mut().flush().await.unwrap();
        });

        let connector = NiriConnector::new(path);
        let stream = connector.connect().await.unwrap();
        let mut reader = BufReader::new(stream);
        let mut buf = String::new();

        let err = connector
            .send_event_stream_handshake(&mut reader, &mut buf)
            .await
            .unwrap_err();

        assert!(
            err.to_string()
                .contains("Handshake failed. Did not receive ack from niri")
        );

        server.await.unwrap();
    }
}
