use todex::wayland::{AsInterface, AsOpCode, Interface};
use todex::wayland::display;

use crate::client::ClientMut;

pub use todex::log::*;

pub fn send_message<R: AsInterface + AsOpCode + display::AsDisplay>(msg: R, client: &mut ClientMut) -> R {
    debug!(
        "client#{} -> {}::{}({})",
        client.id,
        msg.interface(),
        R::OPNAME,
        msg.display()
    );
    msg
}

pub fn recv_message<R: AsInterface + AsOpCode + display::AsDisplay>(msg: R, client: &mut ClientMut) -> R {
    debug!(
        "client#{} <- {}::{}({})",
        client.id,
        msg.interface(),
        R::OPNAME,
        msg.display()
    );
    msg
}

pub fn todo_interface<Op: std::fmt::Display>(interface: Interface, op: Op, client: &mut ClientMut) {
    error!(
        "client#{} {}::{} is not yet implemented",
        interface, op, client.id
    );
}

pub fn todo_operation<R: AsOpCode + AsInterface>(req: R, client: &mut ClientMut) {
    error!(
        "client#{} {}::{} is not yet implemented",
        client.id,
        R::OPNAME,
        req.interface(),
    );
}
