#![allow(unused_variables)]
use crate::Trayle;
use smithay::{
    input::{SeatHandler, SeatState},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
};

smithay::delegate_seat!(Trayle);

impl SeatHandler for Trayle {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.frontend.seat_state
    }
}

