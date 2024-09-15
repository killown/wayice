use crate::core::state::{Backend, WayiceState};
use smithay::wayland::compositor::{with_states, CompositorHandler};
use smithay::wayland::shell::xdg::XdgToplevelSurfaceData;
pub use smithay::{
    backend::input::KeyState,
    desktop::{LayerSurface, PopupKind},
    input::{
        keyboard::{KeyboardTarget, KeysymHandle, ModifiersState},
        pointer::{AxisFrame, ButtonEvent, MotionEvent, PointerTarget, RelativeMotionEvent},
        Seat,
    },
    reexports::wayland_server::{backend::ObjectId, protocol::wl_surface::WlSurface, Resource},
    reexports::{
        calloop::Interest,
        wayland_server::{
            protocol::{wl_buffer::WlBuffer, wl_output},
            Client,
        },
        winit::window::{WindowAttributes, WindowId},
    },
    utils::{IsAlive, Serial},
    wayland::seat::WaylandFocus,
};

pub fn get_window_info(wl_surface: &WlSurface) -> String {
    with_states(wl_surface, |states| {
        let role = match states.data_map.get::<XdgToplevelSurfaceData>() {
            Some(data) => data.lock().unwrap(),
            None => return "No XdgToplevelSurfaceData found.".to_string(),
        };

        let title = role.title.as_deref().unwrap_or("No Title");
        let app_id = role.app_id.as_deref().unwrap_or("No App ID");
        let is_modal = role.modal;

        let parent_id = role
            .parent
            .as_ref()
            .map(|parent| format!("{}", parent.id()))
            .unwrap_or_else(|| "None".to_string());

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

// Function to list all toplevel windows
pub fn list_toplevel_windows(toplevel_surfaces: &[WlSurface]) -> Vec<String> {
    toplevel_surfaces
        .iter()
        .map(|surface| get_window_info(surface))
        .collect()
}
