//! Wayland message.
//!
//! The main API of this module is the [`Message`] struct. See its struct documentation for more
//! detail of how it works
//!
//! # Usage
//!
//! `Message` can be used to encode a message. Encoding a message requires object id and message
//! payload. `Message` by default is representing this form.
//!
//! ```
//! # use crate::wayland::primitives::ObjectId;
//! # use crate::wayland::message::Message;
//! # use crate::wayland::wire::EncodePayload;
//! fn encode<T: EncodePayload + AsOpCode>(object_id: ObjectId, payload: T) {
//!     let msg = Message::new(object_id, payload);
//!     let encoded = Encode::encode(&msg);
//! }
//! ```
//!
//! [`AsObjectId`]: crate::wayland::primitives::AsObjectId
pub use opcode::{AsOpCode, OpCode};
pub use message::{Message, WlMessage};

mod opcode;
mod message;
