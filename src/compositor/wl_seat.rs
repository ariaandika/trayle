use wayland::wl_keyboard::{KeymapFormat, RepeatInfo};
use wayland::wl_seat::{self, GetKeyboard, GetPointer, GetTouch};

use crate::compositor::prelude::*;

// TODO: blocker: interface error, checks for seat capability

impl RequestHandler<GetPointer> for Compositor {
    fn handle(&mut self, req: GetPointer, client: &mut ClientMut) -> Result<(), WlError> {
        let _ = client.objects.create(req.pointer)?;
        Ok(())
    }
}

impl RequestHandler<GetKeyboard> for Compositor {
    fn handle(&mut self, req: GetKeyboard, client: &mut ClientMut) -> Result<(), WlError> {
        let wl_keyboard = client.objects.create(req.keyboard)?;
        client.send(self.seat.to_keymap_event(KeymapFormat::XkbV1, &wl_keyboard));
        if let Some(bind) = client
            .binds
            .iter()
            .find(|e| e.interface == Interface::WlSeat)
            && bind.version >= RepeatInfo::SINCE
        {
            client.send(wl_keyboard.repeat_info(50, 160));
        }
        Ok(())
    }
}

impl RequestHandler<GetTouch> for Compositor {
    fn handle(&mut self, req: GetTouch, client: &mut ClientMut) -> Result<(), WlError> {
        let _ = client.objects.create(req.touch)?;
        Ok(())
    }
}

impl RequestHandler<wl_seat::Release> for Compositor {
    fn handle(&mut self, _: wl_seat::Release, _: &mut ClientMut) -> Result<(), WlError> {
        // idk what need to do here, perhaps there can be ref count for the seat instance ?
        Ok(())
    }
}
