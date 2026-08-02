use todex::sys::epoll::{Epoll, EpollEvent};
use todex::collections::buffer::Buffer;

pub use todex::sys::epoll::Interest;

// ===== Event =====

/// The event emmited by [`Poller`].
#[derive(Debug, Clone)]
pub struct Event {
    pub key: u64,
    pub interest: Interest,
}

impl From<EpollEvent> for Event {
    #[inline]
    fn from(value: EpollEvent) -> Self {
        let (key, interest) = value.to_parts();
        Self { key, interest }
    }
}

// ===== Poller =====

/// Poll for resources readiness.
pub struct Poller<'a> {
    epoll: &'a Epoll,
    buf: Buffer<EpollEvent>,
}

impl<'a> Poller<'a> {
    /// Create new `Poller` with specified capacity.
    #[inline]
    pub fn new(capacity: usize, epoll: &'a Epoll) -> Self {
        Self {
            epoll,
            buf: Buffer::with_capacity(capacity),
        }
    }
}

impl Poller<'_> {
    /// Read the next available event.
    ///
    /// Event `key` is value provided from the [`Epoll::add`] calls.
    ///
    /// Returns `None` if no event are available.
    #[inline]
    pub fn next_event(&mut self) -> Option<Event> {
        self.buf.pop_front().map(<_>::into)
    }

    /// Block current thread and wait for events.
    ///
    /// Note that unread events are discarded. Read events using [`Poller::next_event`].
    ///
    /// This method will block until either en event source deliver an event, the call is interupted
    /// by a signal handler, or `timeout` expires, then returns the length of the new events.
    #[inline]
    pub fn wait(&mut self, timeout: Option<u32>) -> usize {
        self.buf.clear();
        let len = self.epoll.wait(self.buf.spare_capacity_mut(), timeout);
        // SAFETY: the syscall guarantee that n element has been initialized
        unsafe { self.buf.set_len(len) };
        len
    }
}
