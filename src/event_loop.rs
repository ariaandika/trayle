use std::io;
use std::os::unix::net::{UnixListener, UnixStream};

use crate::epoll::{Epoll, Interest};
use crate::sigfd::Sigfd;

pub struct EventLoop {
    path: String,
    epoll: Epoll,
    sigfd: Sigfd,
    listener: UnixListener,
    streams: Vec<UnixStream>,
}

#[derive(Debug)]
pub enum EventKind<'a> {
    New(&'a mut UnixStream),
    ReadWrite(&'a mut UnixStream, Interest),
    Close(UnixStream, Interest),
}

const LISTENER_KEY: u64 = 0;
const SIGFD_KEY: u64 = 1;
const KEY_OFFSET: u64 = 2;

impl Drop for EventLoop {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

impl EventLoop {
    pub fn new(path: String) -> io::Result<Self> {
        let listener = UnixListener::bind(&path)?;
        listener.set_nonblocking(true)?;

        let sigfd = Sigfd::new()?;

        let epoll = Epoll::new()?;
        epoll.add_read_interest(LISTENER_KEY, &listener)?;
        epoll.add_read_interest(SIGFD_KEY, &sigfd)?;

        Ok(Self {
            path,
            epoll,
            sigfd,
            listener,
            streams: Vec::with_capacity(32),
        })
    }
}

impl EventLoop {
    /// Get the next event.
    ///
    /// This will block current thread.
    pub fn next_event(&mut self) -> io::Result<Option<EventKind<'_>>> {
        loop {
            let Some((key, interest)) = self.epoll.next_event() else {
                eprintln!("[EPOLL] blocking");
                self.epoll.wait()?;
                eprintln!("[EPOLL] complete");
                continue;
            };
            match key {
                LISTENER_KEY => {
                    let stream = match self.listener.accept() {
                        Ok((stream, _)) => stream,
                        Err(err) => match err.kind() {
                            io::ErrorKind::WouldBlock => continue,
                            _ => return Err(err),
                        },
                    };

                    stream.set_nonblocking(true)?;
                    self.epoll.add_read_interest(self.streams.len() as u64 + KEY_OFFSET, &stream)?;
                    let stream = self.streams.push_mut(stream);

                    break Ok(Some(EventKind::New(stream)));
                },
                SIGFD_KEY => {
                    self.sigfd.read()?;
                    return Ok(None);
                }
                key => {
                    if interest.is_close() {
                        let stream = self.streams.swap_remove((key - KEY_OFFSET) as usize);
                        self.epoll.remove_interest(&stream)?;
                        break Ok(Some(EventKind::Close(stream, interest)));
                    } else {
                        let stream = &mut self.streams[(key - KEY_OFFSET) as usize];
                        break Ok(Some(EventKind::ReadWrite(stream, interest)));
                    }
                }
            }
        }
    }
}
