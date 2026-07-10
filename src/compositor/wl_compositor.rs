use todex::wayland::primitives::Version;
use wl_compositor::*;
use wl_surface::*;
use wl_region::*;

use crate::compositor::prelude::*;
use crate::compositor::traits::{CommitEffect, CommitError};
use crate::surface::{Region, Role, Surface};

// ===== wl_compositor =====

impl MessageHandler<CreateSurface> for Compositor {
    fn handle(&mut self, msg: Msg<CreateSurface>, client: &mut ClientMut) {
        client.objects.create_with(msg, self.surfaces.create());
    }
}

impl MessageHandler<CreateRegion> for Compositor {
    fn handle(&mut self, msg: Msg<CreateRegion>, client: &mut ClientMut) {
        client.objects.create_with(msg, self.regions.create());
    }
}

impl MessageHandler<Release> for Compositor {
    fn handle(&mut self, _: Msg<Release>, _: &mut ClientMut) {}
}

// ===== wl_region =====

macro_rules! into_region {
    ($expr:expr) => {{
        let b = $expr;
        Region {
            x: b.x,
            y: b.y,
            width: b.width,
            height: b.height,
        }
    }};
}

impl MessageHandler<wl_region::Destroy> for Compositor {
    fn handle(&mut self, msg: Msg<wl_region::Destroy>, _: &mut ClientMut) {
        self.regions.remove(msg.handle());
    }
}

impl MessageHandler<Add> for Compositor {
    fn handle(&mut self, msg: Msg<Add>, _: &mut ClientMut) {
        self.regions[msg.handle()].add(into_region!(msg.payload()));
    }
}

impl MessageHandler<Subtract> for Compositor {
    fn handle(&mut self, msg: Msg<Subtract>, _: &mut ClientMut) {
        self.regions[msg.handle()].subtract(into_region!(msg.payload()));
    }
}

// ===== wl_surface =====

const V5: Version = Version::new(5).unwrap();

impl MessageHandler<Attach> for Compositor {
    fn handle(&mut self, msg: Msg<Attach>, client: &mut ClientMut) -> Result<(), UnknownId> {
        let surface = &mut self.surfaces[msg.handle()];
        let version = msg.version();
        match msg.into_payload() {
            Attach { buffer: Some(wl_buffer), x, y } => {
                if !matches!((x, y), (0, 0)) {
                    if version >= V5 {
                        // return Err(wl_surface::Error::InvalidOffset);
                        todo!()
                    } else {
                        surface.offset(x, y);
                    }
                }
                let buffer_handle = client.objects.get_with(wl_buffer)?.handle();
                surface.attach(buffer_handle);
            }
            _ => {
                let _buffer_handle = surface.unattach();
            }
        }
        Ok(())
    }
}

impl MessageHandler<Commit> for Compositor {
    fn handle(&mut self, msg: Msg<Commit>, client: &mut ClientMut) -> Result<(), CommitError> {
        let surface = &mut self.surfaces[msg.handle()];

        if surface.is_configured() {
            surface.commit();
            // TODO: temporary implementation
            if let Some(handle) = surface.release_current_buffer() {
                let wl_buffer = self.buffers[handle].wl_buffer;
                client.send(wl_buffer.release());
            }
            for callback in surface.request_frames() {
                client.send(callback.done(self.start.elapsed().as_millis() as u32));
                client.delete_id(callback);
                client.objects.remove(callback)?;
            }
            Ok(())
        } else {
            surface.set_configured();
            surface.commit();
            match surface.role().expect("not yet handled") {
                Role::XdgToplevel(obj) => self.commit(obj, client),
            }
        }
    }
}

// destructor

impl MessageHandler<wl_surface::Destroy> for Compositor {
    fn handle(
        &mut self,
        msg: Msg<wl_surface::Destroy>,
        _: &mut ClientMut,
    ) -> Result<(), wl_surface::Error> {
        self.surfaces.remove(msg.handle()).map(Surface::destroy)
    }
}

// callback

impl MessageHandler<Frame> for Compositor {
    fn handle(&mut self, msg: Msg<Frame>, client: &mut ClientMut) {
        self.surfaces[msg.handle()].request_frame(client.objects.create(msg));
    }
}

impl MessageHandler<GetRelease> for Compositor {
    fn handle(
        &mut self,
        msg: Msg<GetRelease>,
        client: &mut ClientMut,
    ) -> Result<(), wl_surface::Error> {
        let surface = &mut self.surfaces[msg.handle()];
        if !surface.has_pending_buffer() {
            return Err(wl_surface::Error::NoBuffer);
        }
        surface.request_release(client.objects.create(msg));
        Ok(())
    }
}

// properties

impl MessageHandler<Offset> for Compositor {
    fn handle(&mut self, msg: Msg<Offset>, _: &mut ClientMut) {
        self.surfaces[msg.handle()].offset(msg.x, msg.y);
    }
}

// TODO: differentiate surface coordinate and buffer coordinate

impl MessageHandler<Damage> for Compositor {
    fn handle(&mut self, msg: Msg<Damage>, _: &mut ClientMut) {
        // Note! New clients should not use this request. Instead damage can be posted with
        // `wl_surface::damage_buffer` which uses buffer coordinates instead of surface coordinates.
        self.surfaces[msg.handle()].damage(into_region!(msg.payload()));
    }
}

impl MessageHandler<DamageBuffer> for Compositor {
    fn handle(&mut self, msg: Msg<DamageBuffer>, _: &mut ClientMut) {
        self.surfaces[msg.handle()].damage(into_region!(msg.payload()));
    }
}

impl MessageHandler<SetOpaqueRegion> for Compositor {
    fn handle(&mut self, msg: Msg<SetOpaqueRegion>, _: &mut ClientMut) {
        let surface = &mut self.surfaces[msg.handle()];
        match msg.region {
            Some(opaque) => surface.set_opaque(opaque),
            None => {
                let _ = surface.remove_opaque();
            }
        }
    }
}

impl MessageHandler<SetInputRegion> for Compositor {
    fn handle(&mut self, msg: Msg<SetInputRegion>, _: &mut ClientMut) {
        let surface = &mut self.surfaces[msg.handle()];
        match msg.region {
            Some(input) => surface.set_input(input),
            None => {
                let _ = surface.remove_input();
            }
        }
    }
}

impl MessageHandler<SetBufferTransform> for Compositor {
    fn handle(&mut self, msg: Msg<SetBufferTransform>, _: &mut ClientMut) {
        self.surfaces[msg.handle()].set_transform(msg.transform);
    }
}

impl MessageHandler<SetBufferScale> for Compositor {
    fn handle(&mut self, msg: Msg<SetBufferScale>, _: &mut ClientMut) {
        self.surfaces[msg.handle()].set_scale(msg.scale);
    }
}
