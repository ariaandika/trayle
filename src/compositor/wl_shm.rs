use wayland::interface::wl_shm::{FormatEnum, WlShm};

use crate::compositor::BindEffect;
use crate::compositor::prelude::*;

impl BindEffect<WlShm> for Compositor {
    fn bind(&mut self, wl_shm: Object<WlShm>, client: &mut ClientMut) -> Result<(), WlError> {
        client.send(wl_shm.format(FormatEnum::Argb8888));
        client.send(wl_shm.format(FormatEnum::Xrgb8888));
        Ok(())
    }
}
