use wayland::wl_registry::Bind;
use wayland::wl_seat::WlSeat;
use wayland::wl_shm::{PixelFormat, WlShm};

use crate::GLOBALS;
use crate::prelude::*;

impl RequestHandler<Bind<'_>> for Compositor {
    fn handle(&mut self, bind: Bind<'_>, client: &mut ClientMut) -> Result<(), WlError> {
        let Some((bind_name, version, iface)) = GLOBALS.get(bind.name as usize) else {
            return Err(WlError::UnknownBind);
        };
        if bind.id_name != *bind_name {
            return Err(WlError::UnknownBind);
        }
        if bind.id_version > *version {
            return Err(WlError::UnknownBind);
        }
        client.objects.insert_parts(bind.id, *iface, 0)?;

        // some interface has side-effect after binding
        match iface {
            Interface::WlSeat => {
                let seat = bind.create::<WlSeat>();
                client.send(seat.capabilities(self.seat.capability()));
            }
            Interface::WlShm => {
                let shm = bind.create::<WlShm>();
                client.send(shm.format(PixelFormat::Argb8888));
                client.send(shm.format(PixelFormat::Xrgb8888));
            }
            _ => (),
        }

        Ok(())
    }
}

