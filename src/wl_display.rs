use todex::wayland::wl_display::{GetRegistry, Sync};

use crate::GLOBALS;
use crate::prelude::*;

impl RequestHandler<Sync> for Compositor {
    fn handle(&mut self, sync: Sync, client: &mut ClientMut) -> Result<(), WlError> {
        let callback = sync.callback.create();
        client.objects.use_one(&callback)?;
        client.send(callback.done(69));
        client.delete_id(callback);
        Ok(())
    }
}

impl RequestHandler<GetRegistry> for Compositor {
    fn handle(&mut self, request: GetRegistry, client: &mut ClientMut) -> Result<(), WlError> {
        let registry = client.objects.create(request.registry)?;

        for ((iface, version, _), i) in GLOBALS.iter().zip(0..) {
            client.send(registry.global(i, iface, *version));
        }

        Ok(())
    }
}

