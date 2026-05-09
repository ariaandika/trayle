use std::{os::unix::net::UnixListener};
use std::io;

use todex::client::Client;
use todex::wayland::Id;


fn main() -> io::Result<()> {
    let _ = std::fs::remove_file("/tmp/wayland-2");
    let listener = UnixListener::bind("/tmp/wayland-2")?;

    let (stream, _) = listener.accept()?;
    let mut client = Client::new(stream);

    client.read()?;
    let (id, op) = client.peek_message().unwrap();
    println!("unhandled message {id}@{op}");

    client.error(Id::wl_display(), 69, "lmao");
    client.flush()
}

