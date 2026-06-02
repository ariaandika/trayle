use crate::wayland::prelude::*;

simple_object! {
    pub struct WlCallback;
}

impl WlCallback {
    #[inline]
    pub fn done(&self, callback_data: u32) -> Message<Done> {
        Message::new(self, Done { callback_data })
    }
}

// ===== Op =====

opcode! {
    pub enum EventOp {
        Done,
    }
}

// ===== Done =====

pub struct Done {
    callback_data: u32,
}

impl Decode for Done {
    type Output<'a> = Done;

    #[inline]
    fn decode<'a>(decoder: Decoder<'a>) -> Result<Self::Output<'a>, WlError> {
        Ok(Self { callback_data: decoder.read()? })
    }
}

impl Encode for Message<Done> {
    const OPCODE: u16 = EventOp::Done as u16;

    #[inline]
    fn object_id(&self) -> ObjectId {
        self.id()
    }

    #[inline]
    fn encode(self, encoder: Encoder) {
        encoder.encode1(self.callback_data);
    }
}
