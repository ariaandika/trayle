use todex::bitflags::Flags;

use wl_keyboard::{KeymapFormat, RepeatInfo};
use wl_seat::{self, Capability, GetKeyboard, GetPointer, GetTouch, WlSeat};

use crate::compositor::traits::BindEffect;
use crate::compositor::prelude::*;

impl BindEffect<WlSeat> for Compositor {
    fn bind(&mut self, wl_seat: Object<WlSeat>, client: &mut ClientMut) {
        client.send(wl_seat.name(self.seat.name()));
        client.send(wl_seat.capabilities(self.seat.capability()));
    }
}

impl MessageHandler<GetPointer> for Compositor {
    fn handle(&mut self, req: Msg<GetPointer>, client: &mut ClientMut) -> Result<(), wl_seat::Error> {
        if !self.seat.capability().contains(Capability::POINTER) {
            return Err(wl_seat::Error::MissingCapability);
        }
        client.objects.create(req);
        Ok(())
    }
}

impl MessageHandler<GetKeyboard> for Compositor {
    fn handle(&mut self, req: Msg<GetKeyboard>, client: &mut ClientMut) -> Result<(), wl_seat::Error> {
        if !self.seat.capability().contains(Capability::KEYBOARD) {
            return Err(wl_seat::Error::MissingCapability);
        }
        let wl_keyboard = client.objects.create(&req);
        client.send(wl_keyboard.keymap(
            KeymapFormat::XkbV1,
            self.seat.keymap_memfd(),
            self.seat.keymap_size(),
        ));
        if req.version() >= RepeatInfo::SINCE {
            client.send(wl_keyboard.repeat_info(50, 160));
        }
        Ok(())
    }
}

impl MessageHandler<GetTouch> for Compositor {
    fn handle(&mut self, req: Msg<GetTouch>, client: &mut ClientMut) -> Result<(), wl_seat::Error> {
        if !self.seat.capability().contains(Capability::TOUCH) {
            return Err(wl_seat::Error::MissingCapability);
        }
        client.objects.create(req);
        Ok(())
    }
}

impl MessageHandler<wl_seat::Release> for Compositor {
    fn handle(&mut self, _: Msg<wl_seat::Release>, _: &mut ClientMut) {
        // idk what need to do here, perhaps there can be ref count for the seat instance ?
    }
}
