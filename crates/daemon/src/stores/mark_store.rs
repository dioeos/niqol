use std::collections::HashMap;

pub struct MarkStore {
    pub map: HashMap<u8, u64>,
}

impl MarkStore {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}
