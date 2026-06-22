use todex::wayland::display;
use todex::wayland::{Interface, WlError, WlMessage};

use crate::client::ClientMut;

pub use todex::log::*;

pub fn send_message<R: WlMessage + display::AsDisplay>(
    msg: R,
    client: &ClientMut,
) -> R {
    debug!(
        "client#{} -> {}::{}({})",
        client.id,
        msg.interface(),
        R::OPNAME,
        msg.display()
    );
    msg
}

pub fn recv_message<R: WlMessage + display::AsDisplay>(
    msg: R,
    client: &ClientMut,
) -> R {
    debug!(
        "client#{} <- {}::{}({})",
        client.id,
        msg.interface(),
        R::OPNAME,
        msg.display()
    );
    msg
}

pub fn todo_interface<Op: std::fmt::Display>(interface: Interface, op: Op, client: &ClientMut) {
    error!(
        "client#{} {}::{} is not yet implemented",
        interface, op, client.id
    );
}

pub fn todo_operation<R: WlMessage>(req: R, client: &ClientMut) {
    error!(
        "client#{} {}::{} is not yet implemented",
        client.id,
        R::OPNAME,
        req.interface(),
    );
}

pub fn malformed_message(error: WlError, client: &ClientMut) {
    error!("client#{} malformed message: {error}", client.id);
}

pub fn handler_error(interface: Interface, op: u16, error: WlError, client: &ClientMut) {
    error!(
        "client#{} failed to handle {interface}::{op}: {error}",
        client.id
    );
}
