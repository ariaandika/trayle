use torio::conn::WaylandSocket;
use torio::objects::wl_display::Display;
use torio::objects::wl_registry;
use torio::objects::wl_registry::Registry;
use torio::objects::ObjectKind;
use torio::objects::ObjectManager;

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
                match wl_registry::Event::from_message(message).unwrap() {
                    wl_registry::Event::GlobalRemove => {}
                    wl_registry::Event::Global(event) => {
                        println!("[OID:{object_id}] {event:?}");
                    }
                }
            }
            _ => {
                println!("[OID:{object_id}] unhandled message, body: {:?}",tcio::fmt::lossy(&body))
            }
        }
    }

    Ok(())
}

