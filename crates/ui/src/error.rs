use niqol_ipc::error::Error as IpcError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{self:?}")]
    FailedIpcOperation(#[source] niqol_ipc::error::Error),

    #[error("{self:?}")]
    FailedIpcClientConnect(#[source] niqol_ipc::error::Error),

    #[error("{self:?}")]
    UnexpectedSocketConflict(#[source] niqol_ipc::error::Error),
}

impl From<niqol_ipc::error::Error> for Error {
    fn from(value: IpcError) -> Self {
        match value {
            source @ (IpcError::FailedSocketIO(_)
            | IpcError::EmptyIpcResponse(_)
            | IpcError::FailedIpcSerdeOperation(_)) => Self::FailedIpcOperation(source),

            source @ (IpcError::FailedToBindListenerToSocket(_)
            | IpcError::FailedToConnectToIpcSocket(_)
            | IpcError::FailedToFindXdgRuntimeDirVar) => Self::FailedIpcClientConnect(source),

            source
            @ (IpcError::UnexpectedQueryResponse | IpcError::FailedToAcceptConnection(_)) => {
                Self::UnexpectedSocketConflict(source)
            }
        }
    }
}
