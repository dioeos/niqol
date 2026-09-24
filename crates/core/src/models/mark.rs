use crate::Window;
use serde::Serialize;

#[derive(Serialize)]
pub struct Mark {
    pub slot: usize,
    pub window: Window
}
