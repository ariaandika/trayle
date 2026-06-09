use todex::wayland::wl_display::{DeleteId, GetRegistry, Sync};

use crate::GLOBALS;
use crate::prelude::*;

impl RequestHandler<Sync> for Compositor {
    fn handle(&mut self, sync: Sync, client: &mut ClientMut) -> Result<(), WlError> {
        let callback = sync.callback.create();
        client.objects_mut().use_one(&callback)?;
        client.send(callback.done(69));
        client.send(DeleteId::new(&callback));
        Ok(())
    }
}

impl RequestHandler<GetRegistry> for Compositor {
    fn handle(&mut self, request: GetRegistry, client: &mut ClientMut) -> Result<(), WlError> {
        let registry = request.registry.create();
        client.insert(&registry)?;

        for ((iface, version, _), i) in GLOBALS.iter().zip(0..) {
            client.send(registry.global(i, iface, *version));
        }

        Ok(())
    }
}

