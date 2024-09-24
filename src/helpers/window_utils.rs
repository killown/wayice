use crate::shell::WindowElement;
use serde_json::{json, Value};
use smithay::wayland::compositor::with_states;
use smithay::wayland::shell::xdg::XdgToplevelSurfaceData;
use smithay::xwayland::X11Surface;
pub use smithay::{
    backend::input::KeyState,
    desktop::space::{
        constrain_space_element, ConstrainBehavior, ConstrainReference, Space, SpaceRenderElements,
    },
    desktop::{
        space::SpaceElement, utils::OutputPresentationFeedback, Window, WindowSurface, WindowSurfaceType,
    },
    desktop::{LayerSurface, PopupKind},
    input::{
        keyboard::{KeyboardTarget, KeysymHandle, ModifiersState},
        pointer::{AxisFrame, ButtonEvent, MotionEvent, PointerTarget, RelativeMotionEvent},
        Seat,
    },
    output::Output,
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
        // Try to get the role (XdgToplevelSurfaceData) of the surface
        let role = match states.data_map.get::<XdgToplevelSurfaceData>() {
            Some(data) => data.lock().unwrap(),
            None => return json!({"error": "No XdgToplevelSurfaceData found."}).to_string(),
        };

        // Extract relevant information from the surface
        let title = role.title.as_deref().unwrap_or("No Title");
        let app_id = role.app_id.as_deref().unwrap_or("No App ID");
        let is_modal = role.modal;

        let parent_id = role
            .parent
            .as_ref()
            .map(|parent| format!("{}", parent.id()))
            .unwrap_or_else(|| "None".to_string());

        // Create a JSON object with the window information
        let info: Value = json!({
            "title": title,
            "app_id": app_id,
            "is_modal": is_modal,
            "parent_id": parent_id,

        });

        // Convert the JSON object to a string
        info.to_string()
    })
}

pub fn get_x11_window_info(x11_surface: &X11Surface) -> String {
    // Extract relevant information from the X11 surface
    let window_id = format!("{}", x11_surface.window_id());
    let title = x11_surface.title();
    let class = x11_surface.class();
    let instance = x11_surface.instance();
    let pid = x11_surface.pid();

    // Create a JSON object with the window information
    let info = json!({
        "window_id": window_id,
        "title": title,
        "class": class,
        "instance": instance,
        "pid": pid
    });

    // Convert the JSON object to a string
    info.to_string()
}

pub fn list_toplevel_windows(toplevel_surfaces: &[WlSurface]) -> Vec<String> {
    toplevel_surfaces
        .iter()
        .map(|surface| get_window_info(surface))
        .collect()
}
