use wl_shm::{self, CreatePool, FormatEnum, WlShm};
use wl_shm_pool::CreateBuffer;

use crate::compositor::prelude::*;
use crate::shm::ShmPools;

/// An handler after `wl_registry::bind` on `wl_shm`.
pub fn bind(wl_shm: Object<WlShm>, client: &mut ClientMut) {
    client.send(wl_shm.format(FormatEnum::Argb8888));
    client.send(wl_shm.format(FormatEnum::Xrgb8888));
}

// ===== wl_shm =====

pub fn create_pool(
    shm_pool: Msg<CreatePool>,
    client: &mut ClientMut,
    shm_pools: &mut ShmPools,
) -> Result<(), wl_shm::Error> {
    let handle = shm_pools.create_pool(shm_pool.fd, shm_pool.size)?;
    client.objects.create_with(shm_pool, handle);
    Ok(())
}

// ===== wl_shm_pool =====

pub fn create_buffer(
    msg: Msg<CreateBuffer>,
    client: &mut ClientMut,
    res: &mut Resources,
) -> Result<(), wl_shm::Error> {
    let buffer = res.shm_pools.create_buffer(msg.handle(), &msg)?;
    let handle = res.buffers.insert(buffer);
    client.objects.create_with(msg, handle);
    Ok(())
}
