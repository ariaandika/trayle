use std::io;
use std::task::Poll::*;

use epoll::{Epoll, EpollBuf};
use error::Result;
use listener::{Listener, SocketPath};
use macros::try_block;
use sigfd::Sigfd;

// === shared ===
mod macros;
mod error;
// === standard ===
mod epoll;
mod sigfd;
mod listener;
mod net;
// === logic ===
mod wayland;
mod client;
mod clients;

const SOCKET_PATH: SocketPath = SocketPath::new(c"/tmp/wayland-2");

const LISTENER_KEY: u64 = 0;
const SIGFD_KEY: u64 = 1;
const KEY_OFFSET: u64 = 2;

fn main() -> error::Terminate {
    event_loop().into()
}

fn event_loop() -> Result<()> {
    // ===== setup =====

    let listener = Listener::new(SOCKET_PATH)?;
    let sigfd = Sigfd::new()?;
    let mut epoll = Epoll::new()?;

    epoll.add_read_interest(LISTENER_KEY, &listener)?;
    epoll.add_read_interest(SIGFD_KEY, &sigfd)?;

    let mut epoll_buf = EpollBuf::new();
    let mut streams = Vec::with_capacity(8);

    // ===== event loop =====

    loop {
        let Some((key, interest)) = epoll.next_event(&epoll_buf) else {
            eprintln!("[EPOLL] blocking");
            epoll.wait(&mut epoll_buf)?;
            eprintln!("[EPOLL] complete");
            continue;
        };
        match key {
            LISTENER_KEY => {
                let stream = match listener.poll_accept() {
                    Ready(Ok(ok)) => ok,
                    Ready(Err(err)) => {
                        eprintln!("[CLIENT] cannot connect: {err}");
                        continue;
                    },
                    Pending => continue,
                };
                let result = try_block! {
                    stream.set_nonblocking(true)?;
                    epoll.add_read_interest(streams.len() as u64 + KEY_OFFSET, &stream)?;
                    streams.push(stream);
                    Ok::<_, io::Error>(())
                };
                match result {
                    Ok(()) => println!("[CLIENT] connected"),
                    Err(err) => eprintln!("[CLIENT] cannot connect: {err}"),
                }
            }
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
