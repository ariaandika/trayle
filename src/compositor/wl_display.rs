use wl_display::{GetRegistry, Sync};
use wl_registry::Bind;

use crate::compositor::prelude::*;
use crate::compositor::GLOBALS;
use crate::compositor::traits::BindEffect;
use crate::compositor::error::BindError;

impl MessageHandler<Sync> for Compositor {
    fn handle(&mut self, sync: Msg<Sync>, client: &mut ClientMut) {
        let wl_callback = client.objects.use_one(sync.callback_id);
        client.send(wl_callback.done(0));
        client.delete_id(wl_callback);
    }
}

impl MessageHandler<GetRegistry> for Compositor {
    fn handle(&mut self, msg: Msg<GetRegistry>, client: &mut ClientMut) {
        let wl_registry = client.objects.create(msg);
        for (global, i) in GLOBALS.iter().zip(0..) {
            client.send(wl_registry.global(i, global.name(), global.version().to_u32()));
        }
    }
}

impl MessageHandler<Bind<'_>> for Compositor {
    fn handle(&mut self, bind: Msg<Bind<'_>>, client: &mut ClientMut) -> Result<(), BindError> {
        let Some(global) = GLOBALS.get(bind.name as usize) else {
            return Err(BindError::UnknownName);
        };
        if bind.id_name != global.name() {
            return Err(BindError::MissmatchName);
        }
        if bind.id_version > global.version() {
            return Err(BindError::UnsupportedVersion);
        }
        let interface = global.interface();
        client
            .objects
            .insert_parts(bind.new_id, interface, bind.id_version);

        macro_rules! dispatch {
            ($($iface:ident::$bind:ident),*) => {
                match interface {
                    $(Interface::$iface => self.$bind(
                        Object::<$iface>::from_dynamic(bind.new_id, interface),
                        client
                    ),)*
                    _ => {}
                }
            };
        }
        dispatch!(WlSeat::bind, WlShm::bind);

        Ok(())
    }
}
