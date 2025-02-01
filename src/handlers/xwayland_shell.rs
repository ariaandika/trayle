use smithay::wayland::xwayland_shell::{XWaylandShellHandler, XWaylandShellState};

use crate::Trayle;

smithay::delegate_xwayland_shell!(@<B: 'static> Trayle<B>);

impl<B> XWaylandShellHandler for Trayle<B> {
    fn xwayland_shell_state(&mut self) -> &mut XWaylandShellState {
        todo!()
    }
}

