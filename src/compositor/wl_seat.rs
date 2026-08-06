use todex::bitflags::Bitflags;

use wl_keyboard::{KeymapFormat, RepeatInfo};
use wl_seat::{self, Capability, GetKeyboard, GetPointer, GetTouch, WlSeat};

use crate::{compositor::prelude::*, seat::Seat};

/// An handler after `wl_registry::bind` on `wl_seat`.
pub fn bind(wl_seat: Object<WlSeat>, client: &mut ClientMut, seat: &mut Seat) {
    client.send(wl_seat.name(seat.name()));
    client.send(wl_seat.capabilities(seat.capability()));
}

pub fn get_pointer(
    req: Msg<GetPointer>,
    client: &mut ClientMut,
    seat: &mut Seat,
) -> Result<(), wl_seat::Error> {
    if !seat.capability().contains(Capability::POINTER) {
        return Err(wl_seat::Error::MissingCapability);
    }
    client.objects.create(req);
    Ok(())
}

pub fn get_keyboard(
    req: Msg<GetKeyboard>,
    client: &mut ClientMut,
    seat: &mut Seat
) -> Result<(), wl_seat::Error> {
    if !seat.capability().contains(Capability::KEYBOARD) {
        return Err(wl_seat::Error::MissingCapability);
    }
    let wl_keyboard = client.objects.create(&req);
    client.send(wl_keyboard.keymap(
        KeymapFormat::XkbV1,
        seat.keymap_memfd(),
        seat.keymap_size(),
    ));
    if req.version() >= RepeatInfo::SINCE {
        client.send(wl_keyboard.repeat_info(50, 160));
    }
    Ok(())
}

pub fn get_touch(
    req: Msg<GetTouch>,
    client: &mut ClientMut,
    seat: &mut Seat,
) -> Result<(), wl_seat::Error> {
    if !seat.capability().contains(Capability::TOUCH) {
        return Err(wl_seat::Error::MissingCapability);
    }
    client.objects.create(req);
    Ok(())
}
