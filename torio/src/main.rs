use torio::conn::WaylandSocket;
use torio::objects::wl_display::GetRegistry;

fn main() -> anyhow::Result<()> {
    let mut socket = WaylandSocket::connect_default()?;
    let get_registry = GetRegistry::new();
    dbg!(&get_registry);
    socket.send_request(get_registry)?;
    socket.flush()?;

    while let Some(()) = socket.poll_message_debug()? { }

    Ok(())
}
