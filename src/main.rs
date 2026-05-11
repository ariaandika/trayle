use std::io;
use std::os::unix::net::{UnixListener, UnixStream};
use std::task::Poll::{self, *};

use epoll::Epoll;
use error::Result;
use sigfd::Sigfd;

mod net;
mod epoll;
mod sigfd;
mod wayland;
mod client;
mod clients;

mod error;

const SOCKET: &str = "/tmp/wayland-2";

const LISTENER_KEY: u64 = 0;
const SIGFD_KEY: u64 = 1;
const KEY_OFFSET: u64 = 2;

fn main() -> error::Terminate {
    event_loop().into()
}

fn event_loop() -> Result<()> {
    // ===== setup =====

    let _guard = DropGuard;
    let mut listener = UnixListener::bind(SOCKET)?;
    let sigfd = Sigfd::new()?;
    let mut epoll = Epoll::new()?;

    listener.set_nonblocking(true)?;
    epoll.add_read_interest(LISTENER_KEY, &listener)?;
    epoll.add_read_interest(SIGFD_KEY, &sigfd)?;

    let mut streams = Vec::with_capacity(8);

    // ===== event loop =====

    loop {
        let Some((key, interest)) = epoll.next_event() else {
            eprintln!("[EPOLL] blocking");
            epoll.wait()?;
            eprintln!("[EPOLL] complete");
            continue;
        };
        match key {
            LISTENER_KEY => match new_client(&mut listener, &mut streams, &epoll) {
                Ready(Ok(())) => println!("[CLIENT] new"),
                Ready(Err(err)) => eprintln!("[CLIENT] cannot create: {err}"),
                Pending => {}
            },
            SIGFD_KEY => {
                sigfd.read()?;
                break;
            }
            key => {
                if interest.is_close() {
                    let stream = streams.swap_remove((key - KEY_OFFSET) as usize);
                    if let Err(err) = epoll.remove_interest(&stream) {
                        eprintln!("cannot remove epoll interest: {err}");
                    }
                    println!("[CLIENT]: close");
                } else {
                    let stream = &mut streams[(key - KEY_OFFSET) as usize];
                    let mut buf = [0; 1024];
                    let len = io::Read::read(stream, &mut buf)?;
                    println!("[CLIENT]: {:?}", str::from_utf8(&buf[..len]));
                }
            }
        }
    }

    Ok(())
}

fn new_client(
    listener: &mut UnixListener,
    streams: &mut Vec<UnixStream>,
    epoll: &Epoll,
) -> Poll<Result<()>> {
    let stream = match listener.accept() {
        Ok((stream, _)) => stream,
        Err(err) => {
            return match err.kind() {
                io::ErrorKind::WouldBlock => Pending,
                _ => return Ready(Err(err.into())),
            };
        }
    };

    stream.set_nonblocking(true)?;
    epoll.add_read_interest(streams.len() as u64 + KEY_OFFSET, &stream)?;
    streams.push(stream);

    Ready(Ok(()))
}

struct DropGuard;

impl Drop for DropGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(SOCKET);
    }
}
