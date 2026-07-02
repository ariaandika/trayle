use wl_compositor::{CreateSurface, CreateRegion, Release};
use wl_surface::{Attach, Commit, Damage, DamageBuffer, Destroy, Frame, GetRelease, Offset};
use wl_surface::{SetBufferScale, SetBufferTransform, SetInputRegion, SetOpaqueRegion};

use crate::compositor::prelude::*;

impl MessageHandler<CreateSurface> for Compositor {
    fn handle(&mut self, req: Msg<CreateSurface>, client: &mut ClientMut) -> Result<(), WlError> {
        let handle = self.surfaces.create();
        let _ = client.objects.create_handle(req, handle)?;
        Ok(())
    }
}

todo_handler!(CreateRegion);
todo_handler!(Release);

// ===== wl_surface =====

todo_handler!(Destroy);

impl MessageHandler<Attach> for Compositor {
    fn handle(&mut self, msg: Msg<Attach>, client: &mut ClientMut) -> Result<(), WlError> {
        // TODO: buffer storage
        // self.surfaces.get_mut(msg.handle())?.attach(buffer);
        self.todo(msg, client)
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
    fn handle(&mut self, msg: Msg<Frame>, _: &mut ClientMut) -> Result<(), WlError> {
        self.surfaces.get_mut(msg.handle())?.request_frame();
        Ok(())
    }
}

todo_handler!(SetOpaqueRegion);
todo_handler!(SetInputRegion);

impl MessageHandler<Commit> for Compositor {
    fn handle(&mut self, msg: Msg<Commit>, _: &mut ClientMut) -> Result<(), WlError> {
        self.surfaces.get_mut(msg.handle())?.commit();
        Ok(())
    }
}

todo_handler!(SetBufferTransform);
todo_handler!(SetBufferScale);
todo_handler!(DamageBuffer);
todo_handler!(Offset);
todo_handler!(GetRelease);
