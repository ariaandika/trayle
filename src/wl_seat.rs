use wayland::wl_keyboard::KeymapFormat;
use wayland::wl_seat::{self, GetKeyboard, GetPointer, GetTouch};

use crate::prelude::*;

// TODO: blocker: interface error, checks for seat capability

impl RequestHandler<GetPointer> for Compositor {
    fn handle(&mut self, req: GetPointer, client: &mut ClientMut) -> Result<(), WlError> {
        client.objects.insert(&req.pointer)
    }
}

impl RequestHandler<GetKeyboard> for Compositor {
    fn handle(&mut self, req: GetKeyboard, client: &mut ClientMut) -> Result<(), WlError> {
        let wl_keyboard = client.objects.create(req.keyboard)?;
        client.send(self.seat.to_keymap_event(KeymapFormat::XkbV1, &wl_keyboard));
        // TODO: check for bind version
        client.send(wl_keyboard.repeat_info(50, 160));
        Ok(())
    }
}

impl RequestHandler<GetTouch> for Compositor {
    fn handle(&mut self, req: GetTouch, client: &mut ClientMut) -> Result<(), WlError> {
        client.objects.insert(&req.touch)
    }
}

impl RequestHandler<wl_seat::Release> for Compositor {
    fn handle(&mut self, _: wl_seat::Release, _: &mut ClientMut) -> Result<(), WlError> {
        // idk what need to do here, perhaps there can be ref count for the seat instance ?
        Ok(())
    }
}
