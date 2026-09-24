use serde::{Deserialize, Serialize};

use crate::models::Mark;

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum QueryRequest {
    ListMarks,
}

#[derive(Serialize)]
pub enum QueryResponse {
    Marks(Vec<Mark>)
}
