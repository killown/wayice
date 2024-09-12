use crate::compositor;
use wayland_protocols::xdg::xdg_toplevel::XdgToplevelSurfaceData;
use wayland_server::protocol::wl_surface::WlSurface;

pub fn get_window_info(wl_surface: &WlSurface) -> String {
    compositor::with_states(wl_surface, |states| {
        // Attempt to retrieve the XdgToplevelSurfaceData
        let role = match states.data_map.get::<XdgToplevelSurfaceData>() {
            Some(data) => data.lock().unwrap(),
            None => return "No XdgToplevelSurfaceData found.".to_string(),
        };

        // Extract relevant fields from XdgToplevelSurfaceData
        let title = role.title.as_deref().unwrap_or("No Title");
        let app_id = role.app_id.as_deref().unwrap_or("No App ID");
        let is_modal = role.modal;

        // Extract parent surface ID or default to "None"
        let parent_id = role
            .parent
            .as_ref()
            .map(|parent| format!("{}", parent.id())) // Convert to string if parent exists
            .unwrap_or_else(|| "None".to_string());

        // Format the information as a single string
        format!(
            "Title: {} | App ID: {} | Is Modal: {} | Surface ID: {} | Parent: {}",
            title,
            app_id,
            is_modal,
            wl_surface.id(),
            parent_id
        )
    })
}
