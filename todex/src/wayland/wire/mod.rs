//! # Wire Format
//!
//! This API use [`Bytes`] and [`Cmsg`] for memory management. It is a bytes buffer and fds storage.
//! See its documentation for more details.
//!
//! [`Bytes`]: crate::sys::bytes::Bytes
//! [`Cmsg`]: crate::sys::cmsg::Cmsg
//!
//! [`Frame`] is used to decode a message. [`Frame::has_frame`] returns `true` if the buffer
//! contains enough bytes for a frame. Then the buffer can be passed to `Frame` to decode the
//! actual message using [`Decode`] implementation.
//!
//! To encode a message, use the [`Encode`] implementation. Every interface have method to construct
//! its messages wrapped in a [`Message`] to associate it with an object id, which implement the
//! [`Encode`] trait.
//!
//! [`Message`]: crate::wayland::Message
pub use error::DecodeError;
pub use opcode::{AsOpCode, OpCode};
pub use frame::{Frame, FrameError};
pub use read::{Read, Reader};
pub use write::{Write, Writer, Sized2};
pub use decode::{Decode, DecodePayload, Decoder, RawMessage};
pub use encode::{Encode, EncodePayload};

mod error;
mod opcode;
mod frame;
mod read;
mod write;
mod decode;
mod encode;
