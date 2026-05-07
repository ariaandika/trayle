use todex::Id;
use todex::conn::WaylandSocket;
use todex::message::DecodePayload;
use todex::wayland::wl_display::get_registry::GetRegistry;
use todex::wayland::wl_display::{self, sync};
use todex::wayland::{wl_callback, wl_registry};

type BoxError = Box<dyn std::error::Error + Send + Sync>;

fn main() -> Result<(), BoxError> {
    let mut socket = WaylandSocket::connect_default()?;
    println!("Connected");

    let registry_id = 2;
    let callback_id = 3;

    socket.send_request(Id::wl_display(), GetRegistry {
        registry: registry_id,
    });
    socket.send_request(Id::wl_display(), sync::Sync {
        callback: callback_id
    });
    socket.flush()?;
    println!("Send ok");

    while let Some(msg) = socket.poll_message()? {
        let id = msg.object_id();
        let opcode = msg.opcode();
        print!("{id}::{opcode} = ");
        if id == registry_id && opcode == 0 {
            unsafe {
                let global = wl_registry::global::Global::decode_raw(msg.as_ptr())?;
                println!("{global:?}");
            }
        } else if id == callback_id && opcode == 0 {
            unsafe {
                let done = wl_callback::done::Done::decode_raw(msg.as_ptr())?;
                println!("{done:?}");
            }
        } else if id == 1 && opcode == 1 {
            unsafe {
                let delete_id = wl_display::delete_id::DeleteId::decode_raw(msg.as_ptr())?;
                println!("{delete_id:?}");
            }
            break;
        } else {
            println!("{}", tcio::fmt::lossy(&msg.payload()));
        }
    }

    Ok(())
}

