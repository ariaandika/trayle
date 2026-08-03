use std::mem;
use std::os::fd::AsFd;

use todex::sys::epoll::{Epoll, EpollEvent};
use todex::collections::buffer::Buffer;

use crate::error::FatalError;

pub(crate) use todex::sys::epoll::EpollEvent as Event;

// ===== EventKind =====

/// Poll Event Kind.
#[derive(Clone, Copy)]
pub(crate) enum EventKind {
    Client,
    Input,
    Gateway,
    Sigfd,
}

const MSB: u64 = i64::MIN as u64;

impl EventKind {
    fn from_key(key: u64) -> Self {
        if key & MSB == MSB {
            // SAFETY: `key` with `MSB` can only be created from `to_key`
            unsafe { mem::transmute::<u8, Self>(key as u8) }
        } else {
            Self::Client
        }
    }

    fn to_key(self) -> u64 {
        self as u64 | MSB
    }
}

// ===== EventSource =====

/// An Event Source.
pub(crate) trait EventSource: AsFd {
    const KIND: EventKind;
}

macro_rules! impl_source {
    ($vr:ident,$ty:ty) => {
        impl EventSource for $ty {
            const KIND: EventKind = EventKind::$vr;
        }
    };
}
impl_source!(Gateway, crate::client::Gateway);
impl_source!(Input, crate::input::Input);
impl_source!(Sigfd, todex::sys::sigfd::Sigfd);

// ===== Poller =====

const INITIAL_CAPACITY: usize = 128;

/// Poll for resources readiness.
pub(crate) struct Poller {
    epoll: Epoll,
    buf: Buffer<EpollEvent>,
}

impl std::ops::Deref for Poller {
    type Target = Epoll;

    fn deref(&self) -> &Self::Target {
        &self.epoll
    }
}

impl Poller {
    pub fn new() -> Result<Self, FatalError> {
        Ok(Self {
            epoll: Epoll::new()?,
            buf: Buffer::with_capacity(INITIAL_CAPACITY),
        })
    }

    /// Add event source.
    pub fn add_source<S: EventSource>(&self, source: &S) {
        self.epoll.add(S::KIND.to_key(), source);
    }

    /// Read the next available event.
    pub fn next_event(&mut self) -> Option<(Event, EventKind)> {
        self.buf.pop_front().map(|e| {
            (e, EventKind::from_key(e.key))
        })
    }

    /// Block current thread and wait for events.
    ///
    /// Note that unread events are discarded. Read events using [`Poller::next_event`].
    ///
    /// This method will block until either en event source deliver an event, the call is interupted
    /// by a signal handler, or `timeout` expires, then returns the length of the new events.
    pub fn wait(&mut self, timeout: Option<u32>) -> usize {
        self.buf.clear();
        let len = self.epoll.wait(self.buf.spare_capacity_mut(), timeout);
        // SAFETY: the syscall guarantee that n element has been initialized
        unsafe { self.buf.set_len(len) };
        len
    }
}
