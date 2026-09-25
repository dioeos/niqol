use niqol_core::{ActionRequest, QueryRequest};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub enum IpcRequest {
    Action(ActionRequest),
    Query(QueryRequest),
}
