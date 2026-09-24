use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Window {
    pub id: WindowId,
    //"Ghostty"
    pub title: Option<String>,
    //"com.mitchellh.ghostty"
    pub app_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct WindowId(pub u64);
