use std::{env::var_os, ffi::OsString, path::PathBuf};

use anyhow::Context;
use tokio::{io::{AsyncWriteExt, BufWriter}, net::UnixStream};

use crate::action::ActionRequest;

pub async fn dispatch_action(action: ActionRequest) -> Result<(), anyhow::Error> {
    let actions_stream = connect_actions_socket().await?;

    let action_json_payload: String = serde_json::to_string(&action)
        .context("Failed to serialize action request to JSON")?;

    write_paylad_to_actions_socket(actions_stream, action_json_payload).await?;

    Ok(())
}

async fn connect_actions_socket() -> Result<UnixStream, anyhow::Error> {
    let xdg_os_string: OsString = var_os("XDG_RUNTIME_DIR")
        .context("XDG_RUNTIME_DIR environment variable is not set")?;

    let mut action_socket_path = PathBuf::from(xdg_os_string);
    action_socket_path.push("niqol-actions.sock");

    let actions_stream = UnixStream::connect(action_socket_path).await?;

    Ok(actions_stream)
}

async fn write_paylad_to_actions_socket(
    stream: UnixStream,
    json_payload: String
) -> Result<(), anyhow::Error> {
    let mut writer = BufWriter::new(stream);
    writer.write_all(json_payload.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    Ok(())
}
