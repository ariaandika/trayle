use todex::Id;
use todex::conn::WaylandSocket;
use todex::message::DecodePayload;
use todex::wayland::wl_display::GetRegistry;
use todex::wayland::{wl_data_device, wl_data_device_manager, wl_data_source, wl_display};
use todex::wayland::{wl_callback, wl_registry};

type BoxError = Box<dyn std::error::Error + Send + Sync>;

fn main() -> Result<(), BoxError> {
    let mut socket = WaylandSocket::connect_default()?;
    println!("Connected");

    let wl_registry_id = Id::new(2).unwrap();
    let wl_callback_id = 3;

    socket.send(Id::wl_display(), GetRegistry {
        registry: wl_registry_id.as_u32(),
    });
    socket.send(Id::wl_display(), wl_display::Sync {
        callback: wl_callback_id
    });
    socket.flush()?;
    println!("Send ok");

    loop {
        let msg = socket.poll_message()?;
        let id = msg.object_id();
        let opcode = msg.opcode();
        print!("{id}::{opcode} = ");
        if id == wl_registry_id.as_u32() && opcode == 0 {
            unsafe {
                let global = wl_registry::Global::decode_raw(msg.as_ptr())?;
                println!("{global:?}");
            }
        } else if id == wl_callback_id && opcode == 0 {
            unsafe {
                let done = wl_callback::Done::decode_raw(msg.as_ptr())?;
                println!("{done:?}");
            }
        } else if id == 1 && opcode == 1 {
            unsafe {
                let delete_id = wl_display::DeleteId::decode_raw(msg.as_ptr())?;
                println!("{delete_id:?}");
            }
            break;
        } else {
            println!("{}", tcio::fmt::lossy(&msg.payload()));
        }
    }

    let wl_seat_id = Id::new(4).unwrap();
    let wl_data_device_manager_id = Id::new(5).unwrap();
    let wl_data_source_id = Id::new(6).unwrap();
    let wl_data_device_id = Id::new(7).unwrap();

    socket.send(wl_registry_id, wl_registry::Bind {
        name: 1,
        id_name: "wl_seat",
        id_version: 9,
        id: wl_seat_id.as_u32(),
    });
    socket.send(wl_registry_id, wl_registry::Bind {
        name: 2,
        id_name: "wl_data_device_manager",
        id_version: 3,
        id: wl_data_device_manager_id.as_u32(),
    });
    socket.send(wl_data_device_manager_id, wl_data_device_manager::CreateDataSource {
        id: wl_data_source_id.as_u32(),
    });
    socket.send(wl_data_device_manager_id, wl_data_device_manager::GetDataDevice {
        id: wl_data_device_id.as_u32(),
        seat: wl_seat_id.as_u32(),
    });
    socket.send(wl_data_source_id, wl_data_source::Offer {
        mime_type: "text/plain"
    });
    socket.send(wl_data_device_id, wl_data_device::SetSelection {
        source: wl_data_source_id.as_u32(),
        serial: 69,
    });
    socket.flush()?;

    loop {
        let msg = socket.poll_message()?;
        let id = msg.object_id();
        let opcode = msg.opcode();
        print!("{id}::{opcode} = ");
        if id == 1 && opcode == 0 {
            unsafe {
                let error = wl_display::Error::decode_raw(msg.as_ptr())?;
                println!("{error:?}");
            }
        } else if id == wl_data_source_id.as_u32() && opcode == 1 {
            unsafe {
                let send = wl_data_source::Send::decode_raw(msg.as_ptr())?;
                println!("{send:?}");
                let fd = socket.recv_fds_mut().pop().unwrap();
                let mut file = <std::fs::File as std::os::fd::FromRawFd>::from_raw_fd(fd);
                std::io::Write::write_all(&mut file, b"lmao")?;
                std::thread::sleep(std::time::Duration::from_millis(500));
                std::io::Write::write_all(&mut file, b" lmao")?;
                std::thread::sleep(std::time::Duration::from_millis(500));
                std::io::Write::write_all(&mut file, b" lmao")?;
            }
        } else {
            println!("{}", tcio::fmt::lossy(&msg.payload()));
        }
    }
}

