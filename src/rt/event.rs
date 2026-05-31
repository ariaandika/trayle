use std::mem::MaybeUninit;
use std::os::fd::AsRawFd;
use std::ptr::NonNull;
use std::slice;

use crate::collections::alloc;
use crate::sys::epoll::{CreateError, Epoll, EpollEvent, Interest};

const MAX_EVENT: usize = 128;

/// Collections of event sources.
pub struct EventSources {
    epoll: Epoll,
    events: NonNull<MaybeUninit<EpollEvent>>,
    off: usize,
    len: usize,
}

impl EventSources {
    /// Creates new `EventSources`.
    pub fn new() -> Result<EventSources, CreateError> {
        Ok(Self {
            epoll: Epoll::new()?,
            events: alloc::allocate(MAX_EVENT),
            off: 0,
            len: 0,
        })
    }

    /// Add an event source by listening for `read` event from given source's fd.
    ///
    /// `key` is a value that will be returned by `EventSources::wait` when this source emit an
    /// event.
    pub fn add<Fd: AsRawFd>(&self, key: u64, source: &Fd) {
        self.epoll.add(key, source);
    }

    /// Modify event source data.
    ///
    /// This will modifies the `key`, and should also listen for `write` event for given source.
    pub fn modify<Fd: AsRawFd>(&self, key: u64, is_write: bool, source: &Fd) {
        self.epoll.modify(is_write, key, source);
    }

    /// Delete an event source that is added by fd.
    pub fn delete<Fd: AsRawFd>(&self, source: &Fd) {
        self.epoll.delete(source);
    }
}

impl EventSources {
    /// Read the next available event.
    ///
    /// Returns `(key, Interest)`. `key` is value provided from the [`EventSources::add`] calls.
    ///
    /// Returns `None` if no event are available.
    pub fn next_event(&mut self) -> Option<(u64, Interest)> {
        if self.off == self.len {
            return None;
        }
        self.off += 1;
        // SAFETY: checked that `off` is still in bounds
        unsafe {
            Some(
                self.events
                    .add(self.off - 1)
                    .as_ref()
                    .assume_init_ref()
                    .to_parts(),
            )
        }
    }

    /// Block current thread and wait for events.
    ///
    /// Note that unread events are discarded. Read events using [`EventSources::next_event`].
    ///
    /// This method will block until either en event source deliver an event, the call is interupted
    /// by a signal handler, or `timeout` expires.
    pub fn wait(&mut self, timeout: Option<u32>) {
        let events = unsafe { slice::from_raw_parts_mut(self.events.as_ptr(), MAX_EVENT) };
        self.len = self.epoll.wait(events, timeout);
        self.off = 0;
    }
}

impl Iterator for EventSources {
    type Item = (u64, Interest);

    fn next(&mut self) -> Option<Self::Item> {
        self.next_event()
    }
}
