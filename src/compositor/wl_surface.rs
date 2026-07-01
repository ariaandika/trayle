use wayland::interface::wl_surface::{Attach, Commit, Destroy};

use crate::compositor::prelude::*;

impl RequestHandler<Destroy> for Compositor {
    fn handle(&mut self, _: Operation<Destroy>, _: &mut ClientMut) -> Result<(), WlError> {
        Ok(())
    }
}

impl RequestHandler<Attach> for Compositor {
    fn handle(&mut self, _: Operation<Attach>, _: &mut ClientMut) -> Result<(), WlError> {
        Ok(())
    }
}

impl RequestHandler<Commit> for Compositor {
    fn handle(&mut self, _: Operation<Commit>, _: &mut ClientMut) -> Result<(), WlError> {
        Ok(())
    }
}
