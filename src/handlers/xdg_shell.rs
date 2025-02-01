#![allow(unused_variables)]
use crate::Trayle;
use smithay::{
    reexports::wayland_server::protocol::wl_seat::WlSeat,
    utils::Serial,
    wayland::shell::xdg::{
        PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
    },
};

smithay::delegate_xdg_shell!(@<B: 'static> Trayle<B>);

impl<B> XdgShellHandler for Trayle<B> {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        // todo!()
    }

    fn new_popup(&mut self, surface: PopupSurface, positioner: PositionerState) {
        // todo!()
    }

    fn grab(&mut self, surface: PopupSurface, seat: WlSeat, serial: Serial) {
        // todo!()
    }

    fn reposition_request(&mut self, surface: PopupSurface, positioner: PositionerState, token: u32) {
        // todo!()
    }
}

