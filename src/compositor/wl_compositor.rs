use todex::wayland::primitives::Version;
use wl_compositor::*;
use wl_surface::*;
use wl_region::*;

use crate::compositor::prelude::*;
use crate::compositor::error::AttachError;
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
    fn handle(&mut self, msg: Msg<Attach>, client: &mut ClientMut) -> Result<(), AttachError> {
        let surface = self.surfaces[msg.handle()].pending_mut();
        let version = msg.version();
        let Attach { buffer, x, y } = msg.into_payload();

        surface.buffer = match buffer {
            Some(obj) => Some(client.objects.get_with(obj)?.handle()),
            None => None,
        };

        if !matches!((x, y), (0, 0)) {
            // version is 5 or higher, passing any non-zero x or y is a protocol violation, and will
            // result in an 'invalid_offset' error being raised.
            if version >= V5 {
                return Err(wl_surface::Error::InvalidOffset.into());
            }
            surface.offset.0 += x;
            surface.offset.1 += y;
        }

        Ok(())
    }
}

impl MessageHandler<Commit> for Compositor {
    fn handle(&mut self, msg: Msg<Commit>, client: &mut ClientMut) -> Result<(), CommitError> {
        let surface = &mut self.surfaces[msg.handle()];
        surface.swap_state();

        let is_configured = surface.is_configured();
        if is_configured {
            if let Some(handle) = surface.current_mut().buffer.take() {
                // TODO: temporary implementation, write surface as ppm file
                let buffer = &mut self.buffers[handle];
                let shm_pool = match buffer.factory {
                    crate::shm::BufferFactory::ShmPool(handle) => &mut self.shm_pools[handle],
                };
                let pixels = shm_pool.as_slice();
                let mut file = std::fs::File::create("/tmp/img.ppm").unwrap();
                std::io::Write::write_all(&mut file, b"P6\n1280 720\n255\n").unwrap();
                for y in 0..buffer.height {
                    let row = &pixels[(y * buffer.stride) as usize..];

                    for x in 0..buffer.width {
                        let pixel = &row[(x * 4) as usize..(x * 4 + 4) as usize];

                        let b = pixel[0];
                        let g = pixel[1];
                        let r = pixel[2];

                        std::io::Write::write_all(&mut file, &[r, g, b]).unwrap();
                    }
                }

                let wl_buffer = buffer.wl_buffer;
                client.send(wl_buffer.release());
            }

            let timestamp = self.start.elapsed().as_millis() as u32;
            if let Some(callback) = surface.current_mut().request_frames.take() {
                client.send(callback.done(timestamp));
                client.delete_id(callback);
                client.objects.remove(callback)?;
            }
        } else {
            surface.set_configured();
        }

        if let Some(role) = surface.role() {
            match role {
                Role::XdgToplevel(obj) => self.commit(is_configured, obj, client)?,
            }
        }

        Ok(())
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
        // TODO: wl_surface: handle stacking frame requests
        let surface = self.surfaces[msg.handle()].pending_mut();
        surface.request_frames = Some(client.objects.create(msg));
    }
}

impl MessageHandler<GetRelease> for Compositor {
    fn handle(
        &mut self,
        msg: Msg<GetRelease>,
        client: &mut ClientMut,
    ) -> Result<(), wl_surface::Error> {
        // TODO: wl_surface: handle stacking release requests
        let surface = self.surfaces[msg.handle()].pending_mut();
        if surface.buffer.is_none() {
            return Err(wl_surface::Error::NoBuffer);
        }
        surface.request_release = Some(client.objects.create(msg));
        Ok(())
    }
}

// properties

impl MessageHandler<Offset> for Compositor {
    fn handle(&mut self, msg: Msg<Offset>, _: &mut ClientMut) {
        let (cr_x, cr_y) = self.surfaces[msg.handle()].current().offset;
        let pending = self.surfaces[msg.handle()].pending_mut();

        // The x and y arguments specify the location of the new pending buffer's upper left corner,
        // relative to the current buffer's upper left corner
        pending.offset.0 = cr_x + msg.x;
        pending.offset.1 = cr_y + msg.y;
    }
}

impl MessageHandler<Damage> for Compositor {
    fn handle(&mut self, msg: Msg<Damage>, _: &mut ClientMut) {
        // Note! New clients should not use this request. Instead damage can be posted with
        // `wl_surface::damage_buffer` which uses buffer coordinates instead of surface coordinates.
        self.surfaces[msg.handle()]
            .pending_mut()
            .damage
            .union(into_region!(msg.payload()))
    }
}

impl MessageHandler<DamageBuffer> for Compositor {
    fn handle(&mut self, msg: Msg<DamageBuffer>, _: &mut ClientMut) {
        // TODO: differentiate surface coordinate and buffer coordinate
        self.surfaces[msg.handle()]
            .pending_mut()
            .damage
            .union(into_region!(msg.payload()))
    }
}

impl MessageHandler<SetOpaqueRegion> for Compositor {
    fn handle(&mut self, msg: Msg<SetOpaqueRegion>, _: &mut ClientMut) {
        let surface = self.surfaces[msg.handle()].pending_mut();
        surface.opaque = msg.region;
    }
}

impl MessageHandler<SetInputRegion> for Compositor {
    fn handle(&mut self, msg: Msg<SetInputRegion>, _: &mut ClientMut) {
        let surface = self.surfaces[msg.handle()].pending_mut();
        surface.input = msg.region;
    }
}

impl MessageHandler<SetBufferTransform> for Compositor {
    fn handle(&mut self, msg: Msg<SetBufferTransform>, _: &mut ClientMut) {
        let surface = self.surfaces[msg.handle()].pending_mut();
        surface.transform = msg.transform;
    }
}

impl MessageHandler<SetBufferScale> for Compositor {
    fn handle(&mut self, msg: Msg<SetBufferScale>, _: &mut ClientMut) {
        let surface = self.surfaces[msg.handle()].pending_mut();
        surface.scale = msg.scale;
    }
}
