use wl_shm::{FormatEnum, WlShm, CreatePool, Release};

use crate::compositor::traits::BindEffect;
use crate::compositor::prelude::*;

impl BindEffect<WlShm> for Compositor {
    fn bind(&mut self, wl_shm: Object<WlShm>, client: &mut ClientMut) -> Result<(), WlError> {
        client.send(wl_shm.format(FormatEnum::Argb8888));
        client.send(wl_shm.format(FormatEnum::Xrgb8888));
        Ok(())
    }
}

impl MessageHandler<CreatePool> for Compositor {
    fn handle(&mut self, msg: Msg<CreatePool>, client: &mut ClientMut) -> Result<(), WlError> {
        self.todo(msg, client)
    }
}

impl MessageHandler<Release> for Compositor {
    fn handle(&mut self, msg: Msg<Release>, client: &mut ClientMut) -> Result<(), WlError> {
        self.todo(msg, client)
    }
}

