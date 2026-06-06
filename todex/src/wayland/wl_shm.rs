use crate::wayland::prelude::*;

#[derive(Interface, Debug)]
pub struct WlShm {
    id: ObjectId,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    CreatePool,
    Release,
}

#[derive(Message, Debug)]
#[request(WlShm)]
pub struct CreatePool {
    /// TODO: <wl_shm_pool>
    pub id: ObjectId,
    #[fd]
    pub fd: i32,
    pub size: i32,
}
