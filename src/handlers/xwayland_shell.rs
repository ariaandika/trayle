use smithay::wayland::xwayland_shell::{XWaylandShellHandler, XWaylandShellState};

use crate::{state::BackendState, Trayle};

smithay::delegate_xwayland_shell!(@<B: BackendState + 'static> Trayle<B>);

impl<B> XWaylandShellHandler for Trayle<B> where B: BackendState + 'static {
    fn xwayland_shell_state(&mut self) -> &mut XWaylandShellState {
        todo!()
    }
}

