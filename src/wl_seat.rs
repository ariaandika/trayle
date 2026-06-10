use wayland::wl_seat::GetKeyboard;

use crate::prelude::*;

impl RequestHandler<GetKeyboard> for Compositor {
    fn handle(&mut self, req: GetKeyboard, client: &mut ClientMut) -> Result<(), WlError> {
        let keyboard = req.keyboard.create();
        client.objects.insert(&keyboard)?;
        client.send(self.seat.to_keymap_event(&keyboard));
        Ok(())
    }
}

