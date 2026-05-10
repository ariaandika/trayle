#![allow(clippy::new_without_default)]
mod net;
mod epoll;
mod sigfd;

pub mod wayland;

pub mod client;
pub mod clients;

pub mod event_loop;
