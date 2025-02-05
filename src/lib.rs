//! Trayle, a wayland compositor
//!
//! [`Trayle`] state is build upon 3 structs: [`Config`], [`Frontend`] and [`Backend`]
//!
//! most of the logic contains in [`trayle`] module
//!
//! [`Config`]: config::Config
//! [`Frontend`]: frontend::Frontend
//! [`Backend`]: backend::Backend
pub mod trayle;
pub mod config;
pub mod backend;
pub mod frontend;

pub mod utils;

mod handlers;
// mod input_handler;

// mod render;
// mod shell;
// mod drawing;
// mod cursor;

pub use trayle::Trayle;
