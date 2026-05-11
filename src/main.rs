use std::task::Poll::*;

use epoll::{Epoll, EpollBuf};
use error::Result;
use listener::{Listener, SocketPath};
use sigfd::Sigfd;

// === shared ===
mod macros;
mod error;
// === standard ===
mod epoll;
mod sigfd;
mod net;
mod listener;
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
                while let Ready(result) = listener.poll_accept() {
                    let stream = match result {
                        Ok(ok) => ok,
                        Err(err) => {
                            eprintln!("[CLIENT] cannot connect: {err}");
                            continue;
                        }
                    };
                    if let Err(err) =
                        epoll.add_read_interest(streams.len() as u64 + KEY_OFFSET, &stream)
                    {
                        eprintln!("[CLIENT] cannot add to epoll: {err}");
                        continue;
                    }
                    streams.push(stream);
                    println!("[CLIENT] connected");
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
                    while let Ready(result) = stream.poll_read() {
                        if let Err(err) = result {
                            eprintln!("[CLIENT] failed to read: {err}");
                            break;
                        }
                        println!("[CLIENT]: {:?}", str::from_utf8(stream.read_buffer()));
                        unsafe {
                            let ptr = stream.spare(12);
                            ptr.copy_from_nonoverlapping(b"Hello World!".as_ptr(), 12);
                        }
                        let _ = stream.poll_flush();
                    }
                }
            }
        }
    }

    Ok(())
}
