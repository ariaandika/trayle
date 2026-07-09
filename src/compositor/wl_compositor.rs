use wl_compositor::*;
use wl_surface::*;
use wl_surface::Error as SurfaceError;

use crate::compositor::prelude::*;
use crate::compositor::traits::{CommitEffect, CommitError};
use crate::surface::Role;

// ===== wl_compositor =====

impl MessageHandler<CreateSurface> for Compositor {
    fn handle(&mut self, req: Msg<CreateSurface>, client: &mut ClientMut) {
        client.objects.create_with(req, self.surfaces.create());
    }
}

impl MessageHandler<CreateRegion> for Compositor {
    fn handle(&mut self, _: Msg<CreateRegion>, _: &mut ClientMut) -> Todo<CreateRegion> {
        Todo::new()
    }
}

impl MessageHandler<Release> for Compositor {
    fn handle(&mut self, _: Msg<Release>, _: &mut ClientMut) { }
}

// ===== wl_surface =====

impl MessageHandler<Destroy> for Compositor {
    fn handle(&mut self, msg: Msg<Destroy>, _: &mut ClientMut) -> Result<(), SurfaceError> {
        let surface = &mut self.surfaces[msg.handle()];
        if !surface.is_role_removed() {
            return Err(SurfaceError::DefunctRoleObject);
        }
        self.surfaces.remove(msg.handle());
        Ok(())
    }
}

impl MessageHandler<Attach> for Compositor {
    fn handle(&mut self, msg: Msg<Attach>, client: &mut ClientMut) -> Result<(), ObjectError> {
        let buffer_handle = match msg.buffer {
            Some(buffer) => Some(client.objects.get_with(buffer)?.handle()),
            None => None,
        };
        self.surfaces[msg.handle()].attach(buffer_handle);
        Ok(())
    }
}

impl MessageHandler<Damage> for Compositor {
    fn handle(&mut self, msg: Msg<Damage>, _: &mut ClientMut) {
        self.surfaces[msg.handle()].damage(msg.into_payload());
    }
}

impl MessageHandler<Frame> for Compositor {
    fn handle(&mut self, msg: Msg<Frame>, client: &mut ClientMut) {
        self.surfaces[msg.handle()].request_frame(client.objects.create(msg));
    }
}

todo_handler!(SetOpaqueRegion);
todo_handler!(SetInputRegion);

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

todo_handler!(SetBufferTransform);
todo_handler!(SetBufferScale);
todo_handler!(DamageBuffer);
todo_handler!(Offset);
todo_handler!(GetRelease);
