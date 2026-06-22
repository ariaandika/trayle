use wayland::wl_shm::{PixelFormat, WlShm};

use crate::compositor::BindEffect;
use crate::compositor::prelude::*;

impl BindEffect<WlShm> for Compositor {
    fn bind(&mut self, wl_shm: WlShm, client: &mut ClientMut) -> Result<(), WlError> {
        client.send(wl_shm.format(PixelFormat::Argb8888));
        client.send(wl_shm.format(PixelFormat::Xrgb8888));
        Ok(())
    }
}
