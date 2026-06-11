use crate::prelude::*;

use wayland::wl_surface::Commit;

impl RequestHandler<Commit> for Compositor {
    fn handle(&mut self, _: Commit, _: &mut ClientMut) -> Result<(), WlError> {
        Ok(())
    }
}
