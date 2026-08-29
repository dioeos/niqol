pub(crate) fn from_niri_window(niri_window: niri_ipc::Window) -> niqol_core::Window {
    niqol_core::Window {
        id: niqol_core::WindowId(niri_window.id),
        title: niri_window.title,
        app_id: niri_window.app_id,
    }
}

#[cfg(test)]
mod tests {
    use niri_ipc::WindowLayout;
    use super::*;

    #[test]
    fn from_niri_window_converts_window_fields() {
        let niri_window = niri_ipc::Window {
            id: 42,
            title: Some("Ghostty".to_owned()),
            app_id: Some("com.mitchellh.ghostty".to_owned()),
            pid: Some(5),
            workspace_id: Some(2),
            is_focused: false,
            is_floating: false,
            is_urgent: false,
            layout: WindowLayout {
                pos_in_scrolling_layout: None,
                tile_size: (0.0, 0.0),
                window_size: (0, 0),
                tile_pos_in_workspace_view: None,
                window_offset_in_tile: (0.0, 0.0),
            },
            focus_timestamp: None
        };

        let window = from_niri_window(niri_window);

        assert_eq!(window.id, niqol_core::WindowId(42));
        assert_eq!(window.title.as_deref(), Some("Ghostty"));
        assert_eq!(window.app_id.as_deref(), Some("com.mitchellh.ghostty"));
    }
}
