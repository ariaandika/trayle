use std::fmt;

use todex::log;
use todex::wayland::primitives::ObjectId;
use todex::wayland::object::{OccupiedNewId, UnknownId};
use todex::wayland::message::WlMessage;
use todex::wayland::interface::wl_display::DisplayError;
use todex::wayland::wire::DecodeError;

use crate::client::ClientMut;

macro_rules! impl_from {
    (impl $me:ty; $($vr:ident, $ty:ty;)*) => {$(
        impl From<$ty> for $me {
            fn from(v: $ty) -> Self {
                Self::$vr(v)
            }
        }
    )*};
}

pub use todex::wayland::error::WlError;

pub(super) use impl_from;

// ===== HandleResult =====

/// Handler result.
pub trait HandleResult: Sized {
    fn handle_result(self, id: ObjectId, client: &mut ClientMut);
}

impl HandleResult for () {
    fn handle_result(self, _: ObjectId, _: &mut ClientMut) { }
}

impl<E: WlError + fmt::Display> HandleResult for Result<(), E> {
    fn handle_result(self, id: ObjectId, client: &mut ClientMut) {
        if let Err(err) = self {
            log::error!("client#{} failed to handle request: {err}", client.id);
            client.send_error(id, err);
            client.disconnect();
        }
    }
}

impl<M: WlMessage> HandleResult for Todo<M> {
    fn handle_result(self, id: ObjectId, client: &mut ClientMut) {
        use todex::wayland::interface::WlInterface;
        log::error!(
            "client#{} {}::{} is not yet implemented",
            client.id,
            <M::WlInterface as WlInterface>::INTERFACE_NAME,
            M::OPNAME,
        );
        client.send_error(id, DisplayError::Implementation);
        client.disconnect();
    }
}

// ===== todo marker =====

pub struct Todo<M>(std::marker::PhantomData<fn() -> M>);

impl<M, A, B> From<todex::wayland::message::Message<M, A, B>> for Todo<M> {
    fn from(_: todex::wayland::message::Message<M, A, B>) -> Self {
        Self(std::marker::PhantomData)
    }
}

// ===== MessageError =====

pub enum MessageError {
    UnknownId(UnknownId),
    OccupiedNewId(OccupiedNewId),
    Decode(DecodeError),
    #[expect(dead_code)]
    Disconnect,
}

impl WlError for MessageError {
    fn code(&self) -> u32 {
        match self {
            Self::UnknownId(err) => err.code(),
            Self::OccupiedNewId(err) => err.code(),
            Self::Decode(err) => err.code(),
            Self::Disconnect => 0,
        }
    }

    fn message(&self) -> &str {
        match self {
            Self::UnknownId(err) => err.message(),
            Self::OccupiedNewId(err) => err.message(),
            Self::Decode(err) => err.message(),
            Self::Disconnect => "client disconnect",
        }
    }
}

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownId(err) => err.fmt(f),
            Self::OccupiedNewId(err) => err.fmt(f),
            Self::Decode(err) => err.fmt(f),
            Self::Disconnect => Ok(()),
        }
    }
}

impl_from! {
    impl MessageError;
    Decode, DecodeError;
    UnknownId, UnknownId;
    OccupiedNewId, OccupiedNewId;
}

// ===== BindError =====

#[derive(Debug, Clone, Copy)]
pub enum BindError {
    /// Unknown bind name.
    UnknownName,
    /// Missmatch bind name.
    MissmatchIdName,
    /// Unsupported bind version.
    UnsupportedVersion,
}

impl WlError for BindError {
    fn code(&self) -> u32 {
        DisplayError::InvalidMethod as u32
    }

    fn message(&self) -> &'static str {
        match self {
            Self::UnknownName => "unknown bind name",
            Self::MissmatchIdName => "missmatch bind id name",
            Self::UnsupportedVersion => "unsupported bind version"
        }
    }
}

impl fmt::Display for BindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to bind global: {}", self.message())
    }
}

// ===== CommitError =====

#[derive(Debug, Clone, Copy)]
pub enum CommitError {

}

impl WlError for CommitError {
    fn code(&self) -> u32 {
        match *self { }
    }

    fn message(&self) -> &'static str {
        match *self { }
    }
}

impl fmt::Display for CommitError {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self { }
    }
}

impl From<UnknownId> for CommitError {
    fn from(_: UnknownId) -> Self {
        panic!("dangling id on commit")
    }
}

// ===== AttachError =====

