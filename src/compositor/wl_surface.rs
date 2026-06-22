use wayland::wl_surface::Commit;

use crate::compositor::prelude::*;

impl RequestHandler<Commit> for Compositor {
    fn handle(&mut self, _: Operation<Commit>, _: &mut ClientMut) -> Result<(), WlError> {
        Ok(())
    }
}
