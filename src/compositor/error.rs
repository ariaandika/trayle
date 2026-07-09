use std::assert_matches;
use todex::log;
use todex::wayland::primitives::ObjectId;
use todex::wayland::object::ObjectError;
use todex::wayland::interface::wl_display::DisplayError;
use todex::wayland::wire::DecodeError;

use crate::client::ClientMut;
use crate::compositor::{ClientStatus, ClientStatus as S};

pub use todex::wayland::error::WlError;

// ===== HandleResult =====

/// Handler result.
pub trait HandleResult: Sized {
    fn handle_result(self, id: ObjectId, client: &mut ClientMut) -> ClientStatus;
}

impl HandleResult for () {
    fn handle_result(self, _: ObjectId, _: &mut ClientMut) -> ClientStatus {
        S::Ok
    }
}

impl HandleResult for ClientStatus {
    fn handle_result(self, _: ObjectId, _: &mut ClientMut) -> ClientStatus {
        self
    }
}

impl<E: WlError + std::fmt::Display> HandleResult for Result<(), E> {
    fn handle_result(self, id: ObjectId, client: &mut ClientMut) -> ClientStatus {
        match self {
            Ok(()) => S::Ok,
            Err(err) => {
                log::error!("client#{} failed to handle request: {err}", client.id);
                client.send_error(id, err.code(), err.message());
                S::Disconnect
            }
        }
    }
}

impl<M: todex::wayland::WlMessage> HandleResult for Todo<M> {
    fn handle_result(self, id: ObjectId, client: &mut ClientMut) -> ClientStatus {
        use todex::wayland::interface::WlInterface;
        log::error!(
            "client#{} {}::{} is not yet implemented",
            <M::WlInterface as WlInterface>::INTERFACE_NAME,
            M::OPNAME,
            client.id,
        );
        client.send_error(id, 3, "not yet implemented");
        S::Disconnect
    }
}

// ===== todo marker =====

pub struct Todo<M>(std::marker::PhantomData<fn() -> M>);

impl<M> Todo<M> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

// ===== MessageError =====

pub enum MessageError {
    Object(ObjectError),
    Decode(DecodeError),
}

impl WlError for MessageError {
    fn code(&self) -> u32 {
        match self {
            Self::Object(err) => err.code(),
            Self::Decode(err) => err.code(),
        }
    }

    fn message(&self) -> &str {
        match self {
            Self::Object(err) => err.message(),
            Self::Decode(err) => err.message(),
        }
    }
}

impl From<DecodeError> for MessageError {
    fn from(v: DecodeError) -> Self {
        Self::Decode(v)
    }
}

impl From<ObjectError> for MessageError {
    fn from(v: ObjectError) -> Self {
        Self::Object(v)
    }
}

impl std::fmt::Display for MessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Object(err) => err.fmt(f),
            Self::Decode(err) => err.fmt(f),
        }
    }
}

// ===== BindError =====

#[derive(Debug, Clone, Copy)]
pub enum BindError {
    /// Unknown bind name.
    UnknownName,
    /// Missmatch bind name.
    MissmatchName,
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
            Self::MissmatchName => "missmatch bind name",
            Self::UnsupportedVersion => "unsupported bind version"
        }
    }
}

impl std::fmt::Display for BindError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
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

impl std::fmt::Display for CommitError {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self { }
    }
}

impl From<ObjectError> for CommitError {
    fn from(value: ObjectError) -> Self {
        assert_matches!(value, ObjectError::UnknownId);
        // this is assuming only error from removing object of callback can happens
        panic!("dangling `callback` id")
    }
}
