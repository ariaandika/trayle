#![allow(unused_variables)]
use crate::{state::ClientState, Trayle};
use smithay::{
    reexports::wayland_server::{protocol::wl_surface::WlSurface, Client},
    wayland::compositor::{CompositorClientState, CompositorHandler, CompositorState},
};

smithay::delegate_compositor!(@<B: 'static> Trayle<B>);

impl<B> CompositorHandler for Trayle<B> {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        todo!()
    }
}

