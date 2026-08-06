use wl_display::{GetRegistry, Sync};
use wl_registry::Bind;

use crate::compositor::prelude::*;
use crate::compositor::GLOBALS;
use crate::compositor::error::BindError;
use crate::seat::Seat;

// ===== handler =====

pub fn sync(sync: Msg<Sync>, client: &mut ClientMut) {
    let wl_callback = client.objects.use_one(sync.callback_id);
    client.send(wl_callback.done(0));
    client.delete_id(wl_callback);
}

pub fn get_registry(msg: Msg<GetRegistry>, client: &mut ClientMut) {
    let wl_registry = client.objects.create(msg);
    for (global, i) in GLOBALS.iter().zip(0..) {
        client.send(wl_registry.global(i, global.name(), global.version().to_u32()));
    }
}

pub fn bind(bind: Msg<Bind<'_>>, client: &mut ClientMut, seat: &mut Seat) -> Result<(), BindError> {
    let Some(global) = GLOBALS.get(bind.name as usize) else {
        return Err(BindError::UnknownName);
    };
    if bind.id_name != global.name() {
        return Err(BindError::MissmatchIdName);
    }
    if bind.id_version > global.version() {
        return Err(BindError::UnsupportedVersion);
    }

    let interface = global.interface();
    client
        .objects
        .insert_parts(bind.new_id, interface, bind.id_version);

    use crate::compositor::{wl_seat, wl_shm};
    use Interface as I;

    match interface {
        I::WlSeat => wl_seat::bind(object(bind, interface), client, seat),
        I::WlShm => wl_shm::bind(object(bind, interface), client),
        _ => {}
    }

    Ok(())
}

fn object<I: WlInterface>(bind: Msg<Bind<'_>>, interface: Interface) -> Object<I> {
    Object::from_dynamic(bind.new_id, interface)
}
