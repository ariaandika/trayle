use std::io;

use todex::event_loop::{EventKind, EventLoop};

const SOCKET: &str = "/tmp/wayland-2";

fn main() -> io::Result<()> {
    let mut event_loop = EventLoop::new(SOCKET.into())?;

    while let Some(event) = event_loop.next_event()? {
        match event {
            EventKind::New(stream) => {
                println!("[CLIENT]: new");
                if let Err(err) = io::Write::write_all(stream, b"lmao") {
                    eprintln!("cannot write: {err}");
                }
            }
            EventKind::ReadWrite(stream, interest) => {
                print!("[CLIENT({interest:?})]: ");
                if interest.is_read() {
                    let mut buf = [0; 1024];
                    let len = io::Read::read(stream, &mut buf)?;
                    print!("{:?}", str::from_utf8(&buf[..len]));
                    if let Err(err) = io::Write::write_all(stream, &[b'?'; 4]) {
                        eprintln!("cannot write: {err}");
                    }
                }
                println!();
            }
            EventKind::Close(mut stream, interest) => {
                println!("[CLIENT({interest:?})]: close");
                let _ = io::Write::write_all(&mut stream, b"bye bye");
            }
        }
    }

    println!("closing");
    Ok(())
}
