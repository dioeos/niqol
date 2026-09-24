use crate::Window;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Mark {
    pub slot: usize,
    pub window: Window,
}
