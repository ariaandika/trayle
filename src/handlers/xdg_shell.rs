#![allow(unused_variables)]
use crate::Trayle;
use smithay::{
    desktop::Window,
    reexports::wayland_server::protocol::wl_seat::WlSeat,
    utils::Serial,
    wayland::shell::xdg::{
        PopupSurface, PositionerState, ShellClient, ToplevelSurface, XdgShellHandler, XdgShellState,
    },
};

smithay::delegate_xdg_shell!(Trayle);

impl XdgShellHandler for Trayle {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.frontend.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let window = Window::new_wayland_window(surface);
        self.frontend.space.map_element(window, (0,0), false);
    }

    fn new_popup(&mut self, surface: PopupSurface, positioner: PositionerState) {
        // todo!
        tracing::warn!("popup is not yet implemented");
    }

    fn grab(&mut self, surface: PopupSurface, seat: WlSeat, serial: Serial) {
        // todo!
        tracing::warn!("popup grab is not yet implemented");
    }

    fn reposition_request(&mut self, surface: PopupSurface, positioner: PositionerState, token: u32) {
        // todo!
        tracing::warn!("popup reposition is not yet implemented");
    }

    // provided
    fn new_client(&mut self, client: ShellClient) {
        tracing::debug!("new client via `XdgShellHandler`")
    }
}

