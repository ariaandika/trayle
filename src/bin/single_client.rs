use std::{os::unix::net::UnixListener};
use std::io;

use todex::client::Client;
use todex::wayland::Id;

const SOCKET: &str = "/tmp/wayland-2";

fn main() -> io::Result<()> {
    unsafe { libc::signal(libc::SIGINT, sigint as *const () as usize)};
    let _guard = SocketGuard;

    let listener = UnixListener::bind(SOCKET)?;

    let (stream, _) = listener.accept()?;
    let mut client = Client::new(stream);

    client.read()?;
    let (id, op) = client.peek_message().unwrap();
    println!("unhandled message {id}@{op}");

    client.error(Id::wl_display(), 69, "lmao");
    client.flush()
}

struct SocketGuard;

impl Drop for SocketGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(SOCKET);
    }
}

fn sigint() {
    let _ = std::fs::remove_file(SOCKET);
    std::process::exit(0);
}
