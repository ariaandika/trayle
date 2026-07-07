use wl_compositor::{CreateRegion, CreateSurface, Release};
use wl_surface::{Attach, Commit, Damage, DamageBuffer, Destroy, Frame, GetRelease, Offset};
use wl_surface::{SetBufferScale, SetBufferTransform, SetInputRegion, SetOpaqueRegion};

use crate::compositor::prelude::*;
use crate::compositor::traits::CommitEffect;
use crate::surface::{Role, RoleError};

// ===== wl_compositor =====

impl MessageHandler<CreateSurface> for Compositor {
    fn handle(&mut self, req: Msg<CreateSurface>, client: &mut ClientMut) -> Result<(), WlError> {
        let handle = self.surfaces.create();
        let _ = client.objects.create_with(req, handle)?;
        Ok(())
    }
}

todo_handler!(CreateRegion);
todo_handler!(Release);

// ===== wl_surface =====

todo_handler!(Destroy);

impl MessageHandler<Attach> for Compositor {
    fn handle(&mut self, msg: Msg<Attach>, client: &mut ClientMut) -> Result<(), WlError> {
        let buffer_handle = match msg.buffer {
            Some(buffer) => Some(client.objects.get_with(buffer)?.handle()),
            None => None,
        };
        self.surfaces.get_mut(msg.handle())?.attach(buffer_handle);
        Ok(())
    }
}

impl MessageHandler<Damage> for Compositor {
    fn handle(&mut self, msg: Msg<Damage>, _: &mut ClientMut) -> Result<(), WlError> {
        self.surfaces
            .get_mut(msg.handle())?
            .damage(msg.into_payload());
        Ok(())
    }
}

impl MessageHandler<Frame> for Compositor {
    fn handle(&mut self, msg: Msg<Frame>, client: &mut ClientMut) -> Result<(), WlError> {
        self.surfaces
            .get_mut(msg.handle())?
            .request_frame(client.objects.create(msg)?);
        Ok(())
    }
}

todo_handler!(SetOpaqueRegion);
todo_handler!(SetInputRegion);

impl MessageHandler<Commit> for Compositor {
    fn handle(&mut self, msg: Msg<Commit>, client: &mut ClientMut) -> Result<(), WlError> {
        let surface = self.surfaces.get_mut(msg.handle())?;

        if surface.is_configured() {
            surface.commit();
            // TODO: temporary implementation
            if let Some(handle) = surface.release_current_buffer() {
                let wl_buffer = self.buffers.get_mut(handle).expect("lmao 2").wl_buffer;
                client.send(wl_buffer.release());
            }
            for callback in surface.request_frames() {
                client.send(callback.done(self.start.elapsed().as_millis() as u32));
                client.delete_id(callback);
                client.objects.remove(callback)?;
            }
        } else {
            surface.set_configured();
            surface.commit();
            match surface.role() {
                Role::None => panic!("net yet handled: {}", RoleError::Unset),
                Role::XdgToplevel(obj) => self.commit(obj, client)?,
            }
        }

        Ok(())
    }
}

todo_handler!(SetBufferTransform);
todo_handler!(SetBufferScale);
todo_handler!(DamageBuffer);
todo_handler!(Offset);
todo_handler!(GetRelease);
