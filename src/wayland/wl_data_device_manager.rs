use crate::wayland::prelude::*;
use crate::wayland::wl_data_device::WlDataDevice;
use crate::wayland::wl_data_source::WlDataSource;

// ===== Op =====

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    CreateDataSource,
    GetDataDevice,
    Release,
}

// ===== CreateDataSource =====

#[derive(Debug)]
#[allow(dead_code)]
pub struct CreateDataSource {
    // wl_data_source
    pub data_source: NewId<WlDataSource>,
}

impl Decode for CreateDataSource {
    type Output<'a> = Self;

    #[inline]
    fn decode(decoder: Decoder<'_>) -> Result<Self::Output<'_>, WlError> {
        Ok(Self { data_source: decoder.read()? })
    }
}

impl Encode for Message<CreateDataSource> {
    const OPCODE: u16 = RequestOp::CreateDataSource as u16;

    #[inline]
    fn encode(self, encoder: Encoder) {
        encoder.encode1(self.data_source);
    }
}

// ===== GetDataDevice =====

#[derive(Debug)]
pub struct GetDataDevice {
    // <wl_data_device>
    pub data_device: NewId<WlDataDevice>,
    // <wl_seat>
    pub seat: ObjectId,
}

impl Decode for GetDataDevice {
    type Output<'a> = Self;

    #[inline]
    fn decode(decoder: Decoder<'_>) -> Result<Self::Output<'_>, WlError> {
        let mut reader = decoder.reader();
        Ok(Self {
            data_device: reader.read()?,
            seat: reader.read()?,
        })
    }
}

impl Encode for Message<GetDataDevice> {
    const OPCODE: u16 = RequestOp::GetDataDevice as u16;

    #[inline]
    fn encode(self, encoder: Encoder) {
        encode_me!(encoder, self, data_device, seat);
    }
}
