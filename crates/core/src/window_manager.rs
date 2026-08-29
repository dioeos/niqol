use async_trait::async_trait;

use crate::{Window, WindowId};

#[async_trait]
pub trait WindowManager: Send + Sync {
    async fn get_focused_window(
        &self
    ) -> anyhow::Result<Option<Window>>;

    async fn focus_window(
        &self,
        id: WindowId
    ) -> anyhow::Result<()>;
}
