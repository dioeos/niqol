use std::sync::Arc;

use niqol_core::{MarkService, QueryRequest};


pub(crate) struct QueryHandler {
    mark_service: Arc<MarkService>
}

impl QueryHandler {
    pub(crate) fn new(mark_service: Arc<MarkService>) -> Self {
        Self { mark_service }
    }

    pub(crate) async fn handle_query_request(&self, request: QueryRequest) -> anyhow::Result<()> {
        match request {
            QueryRequest::ListMarks => {
                todo!()
            }
        }
    }
}
