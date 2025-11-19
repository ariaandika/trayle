use torio::conn::WaylandSocket;
use torio::objects::wl_display::GetRegistry;

fn main() -> anyhow::Result<()> {
    let mut socket = WaylandSocket::connect_default()?;
    socket.send_request(GetRegistry::new())?;
    socket.flush()?;
    Ok(())
}
