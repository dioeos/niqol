use std::{env::var_os, ffi::OsString, path::PathBuf};

use niqol_core::{ActionRequest, ActionResponse, Mark, QueryRequest, QueryResponse};
use serde::{Serialize, de::DeserializeOwned};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::UnixStream,
    sync::Mutex,
};

use crate::{error::Error, request::IpcRequest};

pub struct IpcClient {
    ipc_stream: Mutex<UnixStream>,
}

impl IpcClient {
    pub async fn connect() -> Result<Self, Error> {
        let xdg_os_string: OsString =
            var_os("XDG_RUNTIME_DIR").ok_or(Error::FailedToFindXdgRuntimeDirVar)?;

        let mut ipc_socket_path = PathBuf::from(xdg_os_string);
        ipc_socket_path.push("niqol-ipc.sock");

        let ipc_stream = UnixStream::connect(ipc_socket_path)
            .await
            .map_err(Error::FailedToConnectToIpcSocket)?;

        Ok(Self {
            ipc_stream: Mutex::new(ipc_stream),
        })
    }

    pub async fn list_marks(&self) -> Result<Vec<Mark>, Error> {
        match self.request_query(QueryRequest::ListMarks).await? {
            QueryResponse::Marks(marks) => Ok(marks),
            // _ => Err(Error::UnexpectedQueryResponse),
        }
    }

    async fn request_query(&self, query_request: QueryRequest) -> Result<QueryResponse, Error> {
        let response = self
            .request::<IpcRequest, QueryResponse>(IpcRequest::Query(query_request))
            .await?;

        Ok(response)
    }

    pub async fn request_action(
        &self,
        action_request: ActionRequest,
    ) -> Result<ActionResponse, Error> {
        let response = self
            .request::<IpcRequest, ActionResponse>(IpcRequest::Action(action_request))
            .await?;

        Ok(response)
    }

    async fn request<RQ, RS>(&self, request: RQ) -> Result<RS, Error>
    where
        RQ: Serialize,
        RS: DeserializeOwned,
    {
        let payload: String =
            serde_json::to_string(&request).map_err(Error::FailedIpcSerdeOperation)?;
        let mut stream = self.ipc_stream.lock().await;

        stream
            .write_all(payload.as_bytes())
            .await
            .map_err(Error::FailedSocketIO)?;

        stream
            .write_all(b"\n")
            .await
            .map_err(Error::FailedSocketIO)?;

        let mut reader = BufReader::new(&mut *stream);
        let mut response_buf = String::new();

        reader
            .read_line(&mut response_buf)
            .await
            .map_err(Error::EmptyIpcResponse)?;

        serde_json::from_str::<RS>(&response_buf).map_err(Error::FailedIpcSerdeOperation)
    }
}
