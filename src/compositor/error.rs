use todex::log;
use todex::wayland::primitives::ObjectId;
use todex::wayland::error::WlError;

use crate::client::ClientMut;
use crate::compositor::{ClientStatus, ClientStatus as S};

/// Handler result.
pub trait HandleResult: Sized {
    fn handle_result(self, id: ObjectId, client: &mut ClientMut) -> ClientStatus;
}

impl HandleResult for () {
    fn handle_result(self, _: ObjectId, _: &mut ClientMut) -> ClientStatus {
        S::Ok
    }
}

impl HandleResult for Result<(), WlError> {
    fn handle_result(self, id: ObjectId, client: &mut ClientMut) -> ClientStatus {
        match self {
            Ok(()) => S::Ok,
            Err(err) => {
                log::error!("client#{} failed to handle request: {err}", client.id);
                client.send_error(id, err);
                S::Disconnect
            }
        }
    }
}
