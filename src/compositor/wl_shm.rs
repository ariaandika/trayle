use wl_shm::{self, FormatEnum, WlShm, CreatePool, Release as ShmRelease};
use wl_shm_pool::{CreateBuffer, Destroy as ShmPoolDestroy, Resize};
use wl_buffer::Destroy as BufferDestroy;

use crate::compositor::traits::BindEffect;
use crate::compositor::prelude::*;

// ===== wl_shm =====

impl BindEffect<WlShm> for Compositor {
    fn bind(&mut self, wl_shm: Object<WlShm>, client: &mut ClientMut) {
        client.send(wl_shm.format(FormatEnum::Argb8888));
        client.send(wl_shm.format(FormatEnum::Xrgb8888));
    }
}

impl MessageHandler<CreatePool> for Compositor {
    fn handle(&mut self, shm_pool: Msg<CreatePool>, client: &mut ClientMut) -> Result<(), wl_shm::Error> {
        let handle = self.shm_pools.create_pool(shm_pool.fd, shm_pool.size)?;
        client.objects.create_with(shm_pool, handle);
        Ok(())
    }
}

impl MessageHandler<ShmRelease> for Compositor {
    fn handle(&mut self, _: Msg<ShmRelease>, _: &mut ClientMut) {
        // idk what need to do here, perhaps there can be ref count for the shm instance ?
    }
}

// ===== wl_shm_pool =====

impl MessageHandler<CreateBuffer> for Compositor {
    fn handle(&mut self, msg: Msg<CreateBuffer>, client: &mut ClientMut) -> Result<(), wl_shm::Error> {
        let buffer = self.shm_pools.create_buffer(msg.handle(), &msg)?;
        let handle = self.buffers.insert(buffer);
        client.objects.create_with(msg, handle);
        Ok(())
    }
}

impl MessageHandler<ShmPoolDestroy> for Compositor {
    fn handle(&mut self, msg: Msg<ShmPoolDestroy>, _: &mut ClientMut) {
        self.shm_pools.destroy(msg.handle());
    }
}

impl MessageHandler<Resize> for Compositor {
    fn handle(&mut self, msg: Msg<Resize>, _: &mut ClientMut) -> Result<(), wl_shm::Error> {
        self.shm_pools[msg.handle()].resize(msg.size)
    }
}

// ===== wl_buffer =====

impl MessageHandler<BufferDestroy> for Compositor {
    fn handle(&mut self, buffer: Msg<BufferDestroy>, _: &mut ClientMut) {
        let buffer = self.buffers.remove(buffer.handle());
        self.shm_pools.destroy_buffer(buffer);
    }
}
