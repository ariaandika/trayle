#![allow(unused_variables)]
use crate::{trayle::ClientState, Trayle};
use smithay::{
    reexports::wayland_server::{protocol::wl_surface::WlSurface, Client},
    wayland::compositor::{CompositorClientState, CompositorHandler, CompositorState},
    xwayland::XWaylandClientData,
};

smithay::delegate_compositor!(Trayle);

impl CompositorHandler for Trayle {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.frontend.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        if let Some(state) = client.get_data::<ClientState>() {
            return &state.compositor_state;
        }
        if let Some(state) = client.get_data::<XWaylandClientData>() {
            return &state.compositor_state;
        }
        panic!("unknown client data type")
    }

    fn commit(&mut self, surface: &WlSurface) {
        self.surface_commit(surface);
    }
}

