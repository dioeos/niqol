
pub(crate) fn from_niri_window(
    niri_window: niri_ipc::Window
) -> niqol_core::Window {
    niqol_core::Window { 
        id: niqol_core::WindowId(niri_window.id), 
        title: niri_window.title, 
        app_id: niri_window.app_id 
    }
}
