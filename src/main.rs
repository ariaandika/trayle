// this is binary crate, compatibility is not a problem
#![allow(refining_impl_trait)]

// TODO: destructor trait
// TODO: global object trait, for version checking
// TODO: returning interface specific error

use std::process::ExitCode;

// ===== state =====

mod seat;
mod client;
mod rt;

// ===== logic =====

mod compositor;

// ===== util =====

mod log;
mod error;

fn main() -> ExitCode {
    let _guard = log::init();
    <_>::from(rt::event_loop().is_err() as u8)
}
