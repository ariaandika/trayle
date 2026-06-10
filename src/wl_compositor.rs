use wayland::wl_compositor::CreateSurface;

use crate::prelude::*;

impl RequestHandler<CreateSurface> for Compositor {
    fn handle(&mut self, req: CreateSurface, client: &mut ClientMut) -> Result<(), WlError> {
        let surface = req.surface.create();
        client.objects.insert(&surface)
    }
}

