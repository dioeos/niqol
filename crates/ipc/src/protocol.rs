use std::{env::var_os, ffi::OsString, path::PathBuf};

use tokio::{io::AsyncWriteExt, net::UnixStream};

use crate::error::Error::{
    FailedSocketIO, FailedToConnectToIpcSocket, FailedToFindXdgRuntimeDirVar,
};

use super::error::Error;

pub async fn connect_to_ipc_socket() -> Result<UnixStream, Error> {
    let xdg_os_string: OsString = var_os("XDG_RUNTIME_DIR").ok_or(FailedToFindXdgRuntimeDirVar)?;

    let mut ipc_socket_path = PathBuf::from(xdg_os_string);
    ipc_socket_path.push("niqol-ipc.sock");

    let ipc_stream = connect_ipc_socket_at(&ipc_socket_path).await?;

    Ok(ipc_stream)
}

pub async fn write_to_ipc_socket(stream: &mut UnixStream, json: &str) -> Result<(), Error> {
    stream
        .write_all(json.as_bytes())
        .await
        .map_err(FailedSocketIO)?;
    stream.write_all(b"\n").await.map_err(FailedSocketIO)?;
    Ok(())
}

async fn connect_ipc_socket_at(path: &PathBuf) -> Result<UnixStream, Error> {
    UnixStream::connect(path)
        .await
        .map_err(FailedToConnectToIpcSocket)
}
