use std::sync::Arc;

use anyhow::Context;
use tokio::io::BufReader;
use tracing::info;

use crate::NiriConnector;


pub struct NiriListener {
    niri_connector: Arc<NiriConnector>
}

impl NiriListener {
    pub fn new(
        niri_connector: Arc<NiriConnector>
    ) -> Self {
        Self {
            niri_connector
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        info!("listener started");
        let stream = self
            .niri_connector
            .connect()
            .await
            .context("Failed to initialize niri connection")?;

        let mut stream_reader = BufReader::new(stream);
        let mut buf = String::new();

        self.niri_connector
            .send_event_stream_handshake(&mut stream_reader, &mut buf)
            .await
            .context("Failed to initialize niri event stream")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use tempfile::tempdir;
    use tokio::{io::AsyncBufReadExt, net::UnixListener};
    use super::*;

    fn socket_path() -> (tempfile::TempDir, PathBuf) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("niri-test.sock");
        (dir, path)
    }

    #[tokio::test]
    async fn run_fails_when_niri_connection_cannot_be_initialized() {
        let (_dir, path) = socket_path();
        let connector = Arc::new(NiriConnector::new(path));
        let listener = NiriListener::new(connector);

        let err = listener.run().await.unwrap_err();

        assert!(
            err.to_string().contains("Failed to initialize niri connection")
        );
    }

    #[tokio::test]
    async fn run_fails_when_event_stream_cannot_be_initialized() {
        let (_dir, path) = socket_path();
        let listener = UnixListener::bind(&path).unwrap();

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut reader = BufReader::new(stream);

            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();

            //connection drops no reply
            drop(reader);
        });

        let connector = Arc::new(NiriConnector::new(path));
        let listener = NiriListener::new(connector);

        let err = listener.run().await.unwrap_err();
        let chain = format!("{err:#}");

        assert!(chain.contains("Failed to initialize niri event stream"));
        assert!(chain.contains(
                "niri closed the connection before acknowledging EventStream"
        ));

        server.await.unwrap();
    }
}
