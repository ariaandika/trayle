use todex::bitflags::Flags;

use wayland::WlMessage;
use wayland::wl_keyboard::{KeymapFormat, RepeatInfo};
use wayland::wl_seat::{self, Capability, GetKeyboard, GetPointer, GetTouch, WlSeat};

use crate::compositor::BindEffect;
use crate::compositor::prelude::*;

impl BindEffect<WlSeat> for Compositor {
    fn bind(&mut self, wl_seat: WlSeat, client: &mut ClientMut) -> Result<(), WlError> {
        client.send(wl_seat.name(self.seat.name()));
        client.send(wl_seat.capabilities(self.seat.capability()));
        Ok(())
    }
}

impl RequestHandler<GetPointer> for Compositor {
    fn handle(
        &mut self,
        req: Operation<GetPointer>,
        client: &mut ClientMut,
    ) -> Result<(), WlError> {
        if !self.seat.capability().contains(Capability::POINTER) {
            return Err(wl_seat::Error::MissingCapability.into());
        }
        let _ = client.objects.create(req)?;
        Ok(())
    }
}

impl RequestHandler<GetKeyboard> for Compositor {
    fn handle(
        &mut self,
        req: Operation<GetKeyboard>,
        client: &mut ClientMut,
    ) -> Result<(), WlError> {
        if !self.seat.capability().contains(Capability::KEYBOARD) {
            return Err(wl_seat::Error::MissingCapability.into());
        }
        let version = req.version;
        let wl_keyboard = client.objects.create(req)?;
        client.send(self.seat.to_keymap_event(KeymapFormat::XkbV1, &wl_keyboard));
        if version >= RepeatInfo::SINCE {
            client.send(wl_keyboard.repeat_info(50, 160));
        }
        Ok(())
    }
}

impl RequestHandler<GetTouch> for Compositor {
    fn handle(&mut self, req: Operation<GetTouch>, client: &mut ClientMut) -> Result<(), WlError> {
        if !self.seat.capability().contains(Capability::TOUCH) {
            return Err(wl_seat::Error::MissingCapability.into());
        }
        let _ = client.objects.create(req)?;
        Ok(())
    }
}

impl RequestHandler<wl_seat::Release> for Compositor {
    fn handle(&mut self, _: Operation<wl_seat::Release>, _: &mut ClientMut) -> Result<(), WlError> {
        // idk what need to do here, perhaps there can be ref count for the seat instance ?
        Ok(())
    }
}
