use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum QueryRequest {
    ListMarks,
}
