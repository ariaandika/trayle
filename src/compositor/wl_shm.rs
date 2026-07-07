use wl_shm::{FormatEnum, WlShm, CreatePool, Release as ShmRelease};
use wl_shm_pool::{CreateBuffer, Destroy as ShmPoolDestroy, Resize};
use wl_buffer::Destroy as BufferDestroy;

use crate::compositor::traits::BindEffect;
use crate::compositor::prelude::*;

// ===== wl_shm =====

impl BindEffect<WlShm> for Compositor {
    fn bind(&mut self, wl_shm: Object<WlShm>, client: &mut ClientMut) -> Result<(), WlError> {
        client.send(wl_shm.format(FormatEnum::Argb8888));
        client.send(wl_shm.format(FormatEnum::Xrgb8888));
        Ok(())
    }
}

impl MessageHandler<CreatePool> for Compositor {
    fn handle(&mut self, shm_pool: Msg<CreatePool>, client: &mut ClientMut) -> Result<(), WlError> {
        let handle = self.shm_pools.create_pool(shm_pool.fd, shm_pool.size)?;
        client.objects.create_with(shm_pool, handle)?;
        Ok(())
    }
}

impl MessageHandler<ShmRelease> for Compositor {
    fn handle(&mut self, _: Msg<ShmRelease>, _: &mut ClientMut) -> Result<(), WlError> {
        // idk what need to do here, perhaps there can be ref count for the shm instance ?
        Ok(())
    }
}

// ===== wl_shm_pool =====

impl MessageHandler<CreateBuffer> for Compositor {
    fn handle(&mut self, msg: Msg<CreateBuffer>, client: &mut ClientMut) -> Result<(), WlError> {
        let buffer = self.shm_pools.create_buffer(msg.handle(), &msg)?;
        let handle = self.buffers.insert(buffer);
        client.objects.create_with(msg, handle)?;
        Ok(())
    }
}

impl MessageHandler<ShmPoolDestroy> for Compositor {
    fn handle(&mut self, msg: Msg<ShmPoolDestroy>, _: &mut ClientMut) -> Result<(), WlError> {
        self.shm_pools.destroy(msg.handle())?;
        Ok(())
    }
}

impl MessageHandler<Resize> for Compositor {
    fn handle(&mut self, msg: Msg<Resize>, _: &mut ClientMut) -> Result<(), WlError> {
        self.shm_pools.get_mut(msg.handle())?.resize(msg.size)?;
        Ok(())
    }
}

// ===== wl_buffer =====

impl MessageHandler<BufferDestroy> for Compositor {
    fn handle(&mut self, buffer: Msg<BufferDestroy>, _: &mut ClientMut) -> Result<(), WlError> {
        let buffer = self.buffers.remove(buffer.handle())?;
        self.shm_pools.destroy_buffer(buffer)?;
        Ok(())
    }
}
