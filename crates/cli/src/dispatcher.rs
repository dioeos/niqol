use std::{env::var_os, ffi::OsString, path::PathBuf};

use anyhow::Context;
use tracing::debug;
use tokio::{io::{AsyncWriteExt, BufWriter}, net::UnixStream};

use crate::action::ActionRequest;

pub async fn dispatch_action(action: ActionRequest) -> Result<(), anyhow::Error> {
    let actions_stream = connect_actions_socket().await?;

    write_payload_to_actions_socket(actions_stream, &action).await?;

    debug!(?action, "dispatched action");

    Ok(())
}

async fn connect_actions_socket() -> anyhow::Result<UnixStream> {
    let xdg_os_string: OsString = var_os("XDG_RUNTIME_DIR")
        .context("XDG_RUNTIME_DIR environment variable is not set")?;

    let mut action_socket_path = PathBuf::from(xdg_os_string);
    action_socket_path.push("niqol-actions.sock");

    let actions_stream = connect_actions_socket_at(&action_socket_path).await?;

    Ok(actions_stream)
}

async fn connect_actions_socket_at(
    path: &PathBuf
) -> anyhow::Result<UnixStream> {
    UnixStream::connect(path)
        .await
        .with_context(|| {
            format!(
                "Failed to connect to actions socket: {}",
                path.display()
            )
        })
}

async fn write_payload_to_actions_socket(
    stream: UnixStream,
    action: &ActionRequest
) -> Result<(), anyhow::Error> {
    let action_json_payload: String = serde_json::to_string(&action)
        .context("Failed to serialize action request to JSON")?;

    let mut writer = BufWriter::new(stream);
    writer.write_all(action_json_payload.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use tokio::{io::{AsyncBufReadExt, BufReader}, net::UnixListener};

    fn socket_path() -> (tempfile::TempDir, PathBuf) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("niqol-actions.sock");
        (dir, path)
    }

    #[tokio::test]
    async fn connect_actions_socket_at_returns_stream_on_success() {
        let (_dir, path) = socket_path();
        let listener = UnixListener::bind(&path).unwrap();

        let stream = connect_actions_socket_at(&path)
            .await
            .unwrap();

        let (_accepted_stream, _address) = listener
            .accept()
            .await
            .unwrap();

        drop(stream)
    }

    #[tokio::test]
    async fn connect_actions_socket_at_returns_error_when_socket_missing() {
        let (_dir, path) = socket_path();

        let err = connect_actions_socket_at(&path)
            .await
            .unwrap_err();

        assert!(
            err.to_string()
                .contains(&format!(
                        "Failed to connect to actions socket: {}",
                        path.display()
                ))
        );
    }

    #[tokio::test]
    async fn write_payload_to_actions_socket_writes_json() {
        let (client_stream, server_stream) = UnixStream::pair().unwrap();

        let action = ActionRequest::MarkWindow { slot: 1 };

        write_payload_to_actions_socket(server_stream, &action).await.unwrap();

        let mut reader = BufReader::new(client_stream);
        let mut buf = String::new();

        reader
            .read_line(&mut buf)
            .await
            .unwrap();

        let received: ActionRequest= serde_json::from_str(&buf).unwrap();

        assert_eq!(action, received)
    }
}
