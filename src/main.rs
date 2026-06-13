// this is binary crate, compatibility is not a problem
#![allow(refining_impl_trait)]

// TODO: destructor trait
// TODO: global object trait, for version checking
// TODO: returning interface specific error

mod seat;
mod client;

mod compositor;

mod log;
mod error;
mod rt;

use std::process::ExitCode;

fn main() -> ExitCode {
    let _guard = log::init();
    <_>::from(rt::event_loop().is_err() as u8)
}
