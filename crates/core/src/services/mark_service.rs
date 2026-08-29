use std::sync::Arc;

use crate::stores::MarkStore;


pub struct MarkService {
    mark_store: Arc<MarkStore>
}

impl MarkService {
    pub fn new() -> Self {
        Self {
            mark_store: Arc::new(MarkStore::new())
        }
    }
}
