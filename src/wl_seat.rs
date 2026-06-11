use wayland::wl_seat::{GetKeyboard, GetPointer};

use crate::prelude::*;

impl RequestHandler<GetPointer> for Compositor {
    fn handle(&mut self, req: GetPointer, client: &mut ClientMut) -> Result<(), WlError> {
        client.objects.insert(&req.pointer)?;
        Ok(())
    }
}

impl RequestHandler<GetKeyboard> for Compositor {
    fn handle(&mut self, req: GetKeyboard, client: &mut ClientMut) -> Result<(), WlError> {
        let wl_keyboard = client.objects.create(req.keyboard)?;
        client.send(self.seat.to_keymap_event(&wl_keyboard));
        Ok(())
    }
}

