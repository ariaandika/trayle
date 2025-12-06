use torio::conn::WaylandSocket;
use torio::objects::wl_display::Display;
use torio::objects::wl_registry;
use torio::objects::wl_registry::Registry;
use torio::objects::ObjectKind;
use torio::objects::ObjectManager;
use torio::objects::roundup_4;

fn main() -> anyhow::Result<()> {
    let mut socket = WaylandSocket::connect_default()?;

    let mut manager = ObjectManager::new();
    let display = Display::new();
    let registry = Registry::with_manager(&mut manager);

    socket.send_request(display.get_registry(&registry))?;
    socket.flush()?;

    while let Some(message) = socket.poll_message()? {
        let object_id = message.object_id();
        let body = message.body();

        let kind = manager.event_kind(&message).expect("invalid object from server");

        match kind {
            ObjectKind::Display => {
                // TODO: error event
            }
            ObjectKind::Registry => {
                if message.opcode() == wl_registry::EVENT_GLOBAL_CODE {
                    let name = u32::from_ne_bytes(*body.first_chunk::<4>().unwrap());
                    let i_len = u32::from_ne_bytes(*body[4..].first_chunk::<4>().unwrap());
                    let i_str = &body[8..8 + i_len as usize];
                    let version = u32::from_ne_bytes(*body[roundup_4!(8usize + i_len as usize)..].first_chunk::<4>().unwrap());
                    println!(
                        "[OID:{object_id}] name: {name}, interface: {}, version: {version}",
                        tcio::fmt::lossy(&i_str)
                    );
                }
            }
            _ => {
                println!("[OID:{object_id}] unhandled message, body: {:?}",tcio::fmt::lossy(&body))
            }
        }
    }

    Ok(())
}

