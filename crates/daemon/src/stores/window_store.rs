use std::collections::HashMap;

use tokio::sync::Mutex;

pub struct WindowStore {
    pub map: Mutex<HashMap<u64, niri_ipc::Window>>
}

impl WindowStore {
    pub fn new() -> Self {
        Self {
            map: Mutex::new(HashMap::new())
        }
    }
}
