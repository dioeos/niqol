use std::collections::HashMap;

pub struct MarkStore {
    mark_map: HashMap<u8, u64>,
}

impl MarkStore {
    pub fn new() -> Self {
        Self {
            mark_map: HashMap::new(),
        }
    }
}
