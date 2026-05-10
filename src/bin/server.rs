use std::io;

use todex::event_loop::{EventKind, EventLoop};

const SOCKET: &str = "/tmp/wayland-2";

fn main() -> io::Result<()> {
    let _guard = SocketGuard;

    let mut event_loop = EventLoop::new(SOCKET)?;

    'root: loop {
        event_loop.wait_events()?;
        while let Some(event) = event_loop.next_event()? {
            match event {
                EventKind::Incoming(stream) => {
                    println!("new client");
                    let _ = io::Write::write_all(stream, b"lmao");
                }
                EventKind::Sigint => break 'root,
            }
        }
    }

    eprintln!("closing");
    Ok(())
}

struct SocketGuard;

impl Drop for SocketGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(SOCKET);
    }
}
