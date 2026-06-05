use crate::wayland::prelude::*;

// ===== Op =====

opcode! {
    pub enum RequestOp {
        CreatePool,
        Release,
    }
}

// ===== CreatePool =====

#[derive(Debug)]
#[allow(dead_code)]
pub struct CreatePool {
    /// <wl_shm_pool>
    pub id: ObjectId,
    pub fd: i32,
    pub size: i32,
}

impl Decode for CreatePool {
    type Output<'a> = Self;

    #[inline]
    fn decode(mut decoder: Decoder<'_>) -> Result<Self::Output<'_>, WlError> {
        let fd = decoder.pop_fd()?;
        let mut reader = decoder.reader();
        Ok(Self {
            id: reader.read()?,
            fd,
            size: reader.read()?,
        })
    }
}

impl Encode for Message<CreatePool> {
    const OPCODE: u16 = RequestOp::CreatePool as u16;

    #[inline]
    fn encode(self, mut encoder: Encoder) {
        encoder.push_fd(self.fd);
        const SIZE: u16 = const { 8u16 + 4 + 4 };
        unsafe { encoder.encode(SIZE) }
            .write(self.id)
            .write(self.size);
    }
}
