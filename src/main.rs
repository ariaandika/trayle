use std::process::ExitCode;
use std::task::Poll::*;

use todex::sys::buffer::{self, Buffer};
use todex::sys::listener::{Listener, SocketPath};
use todex::sys::sigfd::Sigfd;
use todex::wayland::{self, Decode, Frame, Interface, OpCode, WlError};
use todex::compositor::clients::{ClientId, ClientMut, Clients};
use todex::compositor::seat::Seat;
use todex::rt::poller::Poller;
use todex::log;

const SOCKET_PATH: SocketPath = SocketPath::new(c"/tmp/wayland-2");

const STATIC_KEY_MASK: u64 = i64::MIN as u64;
const LISTENER_KEY: u64 = STATIC_KEY_MASK | 1;
const SIGFD_KEY: u64 = STATIC_KEY_MASK | 2;

fn main() -> ExitCode {
    let _guard = log::init();
    <_>::from(event_loop().is_err() as u8)
}

fn event_loop() -> Result<(), FatalError> {
    let seat = Seat::new()?;
    let listener = Listener::new(SOCKET_PATH)?;
    let sigfd = Sigfd::new()?;

    let mut read_buf = Buffer::new();
    let mut write_buf = Buffer::new();

    let mut clients = Clients::new();
    let mut compositor = Compositor { seat };

    let mut poll = Poller::new()?;

    poll.add(LISTENER_KEY, &listener);
    poll.add(SIGFD_KEY, &sigfd);

    // ===== event loop =====

    loop {
        let Some((key, interest)) = poll.next_event() else {
            log::trace!(target: "polling", "blocking");
            log::flush();
            poll.wait(None);
            continue;
        };

        if key == SIGFD_KEY {
            log::info!("{} signal received", sigfd.read());
            break;
        }

        if key == LISTENER_KEY {
            while let Ready(result) = listener.poll_accept() {
                match result {
                    Ok(fd) => {
                        let (id, client) = clients.insert(fd);
                        poll.add(id.to_u64(), client);
                        log::debug!(target: format_args!("client#{id}"), "connected");
                    }
                    Err(err) => {
                        log::error!(target: "listener", "{err}");
                        break;
                    }
                }
            }
            continue;
        }

        debug_assert!(read_buf.is_empty());
        debug_assert!(write_buf.is_empty());

        let id = ClientId::from_u64(key);

        let Some(client) = clients.get_mut(id) else {
            log::warn!(target: "polling", "unknown client id from event key: {id}");
            continue;
        };

        client.buffer_mut().restore(&mut read_buf, &mut write_buf);

        // cope and seeth: https://github.com/rust-lang/rust/issues/31436
        let result = (|| {
            if interest.is_close() {
                return Err(HandleError);
            }

            if interest.is_read() {
                let mut client = ClientMut::new(id, client, &mut write_buf);
                loop {
                    if Frame::has_frame(&read_buf) {
                        router(&mut read_buf, &mut client, &mut compositor)
                            .inspect_err(|e| client.send_global_error(*e))?;
                    } else {
                        if read_buf.recvmsg(&client)?.is_pending() {
                            break;
                        }
                    }
                }
            }

            if !write_buf.is_empty() {
                let is_pending = write_buf.sendmsg(client)?.is_pending();
                match (is_pending, interest.is_write()) {
                    (true, false) => {
                        // first time write pending, add write interest
                        poll.modify(id.to_u64(), true, client);
                    }
                    (false, true) => {
                        // previous write pending complete, remove write interest
                        poll.modify(id.to_u64(), false, client);
                    }
                    _ => {}
                }
            }

            Ok::<_, HandleError>(())
        })();

        if result.is_ok() {
            if !read_buf.is_empty() || !write_buf.is_empty() {
                log::warn!(
                    target: format_args!("client#{id}"),
                    "partial message read: {}, write: {}",
                    read_buf.len(),
                    write_buf.len()
                );
                // the sad pending bytes cannot stay in shared buffer because it will be used for other
                // socket, it will be stored in on demand allocation
                client.buffer_mut().copy_from(&read_buf, &write_buf);
            }
        } else {
            if !write_buf.is_empty() {
                let _ = write_buf.sendmsg(client);
            }
            poll.delete(client);
            clients.remove(id);
            log::debug!(target: format_args!("client#{id}"), "disconnected");
        }

        read_buf.clear();
        write_buf.clear();
    }

    Ok(())
}

// ===== handler =====

static GLOBALS: [(&str, u32, Interface); 5] = [
    ("wl_compositor", 7, Interface::WlCompositor),
    ("wl_shm", 2, Interface::WlShm),
    ("wl_data_device_manager", 4, Interface::WlDataDeviceManager),
    ("wl_seat", 10, Interface::WlSeat),
    ("xdg_wm_base", 7, Interface::XdgWmBase),
];

pub struct Compositor {
    seat: Seat,
}

pub fn router(
    read_buf: &mut Buffer,
    client: &mut ClientMut,
    compositor: &mut Compositor,
) -> Result<(), WlError> {
    use wayland::interfaces::*;

    let (id, op, frame) = Frame::new(read_buf)?;
    let interface = if id.is_display() {
        Interface::WlDisplay
    } else {
        match client.get_object(id) {
            Some(object) => object.interface(),
            None => return Err(WlError::UnknownObject),
        }
    };

    macro_rules! handle_me {
        (@OP $iface:ident { $($req:ident $($flag:ident)?),* $(,)? }) => {
            match <_>::try_from_op(op)? { $(
                $iface::RequestOp::$req => handle_me!(@CALL $iface $req $($flag)?),
            )* }
        };
        (@CALL $iface:ident $req:ident todo) => {{
            compositor.todo(interface, op, client)
        }};
        (@CALL $iface:ident $req:ident) => {
            compositor.call_handler(interface, $iface::$req::decode_with(frame)?, client)
        };
        ($($iface:ident {$($tt:tt)*})*) => {
            match interface {
                $(
                    Interface::$iface => handle_me!(@OP $iface {$($tt)*}),
                )*
                _ => compositor.todo(interface, op, client),
            }
        };
    }

    handle_me! {
        WlDisplay { Sync, GetRegistry }
        WlRegistry { Bind }
        WlCompositor {
            CreateSurface
        }
        WlShm {
            CreatePool todo,
            Release todo,
        }
        WlSeat {
            GetPointer todo,
            GetKeyboard,
        }
        WlDataDeviceManager {
            CreateDataSource,
            GetDataDevice,
            Release todo,
        }
    }
}

// ===== handlers =====

impl Compositor {
    fn call_handler<Request>(
        &mut self,
        interface: Interface,
        request: Request,
        client: &mut ClientMut,
    ) -> Result<(), WlError>
    where
        Self: RequestHandler<Request>,
        Request: std::fmt::Debug,
    {
        client.log_debug(format_args!("<- {interface}::{request:?}"));
        self.handle(request, client)
    }

    fn todo<Op: std::fmt::Debug>(
        &mut self,
        interface: Interface,
        op: Op,
        client: &mut ClientMut,
    ) -> Result<(), WlError> {
        client.log_error(format_args!("`{interface}::{op:?}` is not yet implemented"));
        WlError::todo()
    }
}

trait RequestHandler<Request>: Sized {
    fn handle(&mut self, request: Request, client: &mut ClientMut) -> Result<(), WlError>;
}

mod wl_display {
    use super::*;
    use wayland::wl_display::{DeleteId, GetRegistry, Sync};

    impl RequestHandler<Sync> for Compositor {
        fn handle(&mut self, sync: Sync, client: &mut ClientMut) -> Result<(), WlError> {
            let callback = sync.callback.create();
            client.objects_mut().use_one(&callback)?;
            client.send(callback.done(69));
            client.send(DeleteId::new(&callback));
            Ok(())
        }
    }

    impl RequestHandler<GetRegistry> for Compositor {
        fn handle(&mut self, request: GetRegistry, client: &mut ClientMut) -> Result<(), WlError> {
            let registry = request.registry.create();
            client.insert(&registry)?;

            for ((iface, version, _), i) in GLOBALS.iter().zip(0..) {
                client.send(registry.global(i, iface, *version));
            }

            Ok(())
        }
    }
}

mod wl_registry {
    use super::*;
    use wayland::wl_registry::Bind;
    use wayland::wl_seat::WlSeat;

    impl RequestHandler<Bind<'_>> for Compositor {
        fn handle(&mut self, bind: Bind<'_>, client: &mut ClientMut) -> Result<(), WlError> {
            let Some((bind_name, version, iface)) = GLOBALS.get(bind.name as usize) else {
                return Err(WlError::UnknownBind);
            };
            if bind.id_name != *bind_name {
                return Err(WlError::UnknownBind);
            }
            if bind.id_version > *version {
                return Err(WlError::UnknownBind);
            }
            client.objects_mut().insert_with(bind.id, *iface, 0)?;

            // some interface has side-effect after binding
            if let Interface::WlSeat = iface {
                let seat = bind.create::<WlSeat>();
                client.send(seat.capabilities(self.seat.capability()));
            }

            Ok(())
        }
    }
}

mod wl_compositor {
    use super::*;
    use wayland::wl_compositor::CreateSurface;

    impl RequestHandler<CreateSurface> for Compositor {
        fn handle(&mut self, req: CreateSurface, client: &mut ClientMut) -> Result<(), WlError> {
            let surface = req.surface.create();
            client.insert(&surface)
        }
    }
}

mod wl_seat {
    use super::*;
    use wayland::wl_seat::GetKeyboard;

    impl RequestHandler<GetKeyboard> for Compositor {
        fn handle(&mut self, req: GetKeyboard, client: &mut ClientMut) -> Result<(), WlError> {
            let keyboard = req.keyboard.create();
            client.insert(&keyboard)?;
            client.send(self.seat.to_keymap_event(&keyboard));
            Ok(())
        }
    }
}

mod wl_data_device_manager {
    use super::*;
    use wayland::wl_data_device_manager::{CreateDataSource, GetDataDevice};

    impl RequestHandler<CreateDataSource> for Compositor {
        fn handle(&mut self, req: CreateDataSource, client: &mut ClientMut) -> Result<(), WlError> {
            let data_source = req.data_source.create();
            client.insert(&data_source)
        }
    }

    impl RequestHandler<GetDataDevice> for Compositor {
        fn handle(&mut self, req: GetDataDevice, client: &mut ClientMut) -> Result<(), WlError> {
            let data_device = req.data_device.create();
            let Some(object) = client.get_object(req.seat) else {
                return Err(WlError::UnknownObject);
            };
            let Interface::WlSeat = object.interface() else {
                return Err(WlError::UnknownObject);
            };
            client.insert(&data_device)
        }
    }
}

// ===== Errors =====

struct HandleError;

impl From<buffer::ReadError> for HandleError {
    fn from(err: buffer::ReadError) -> Self {
        if !err.is_connection_aborted() {
            log::error!("failed to read socket: {err}");
        }
        Self
    }
}

impl From<buffer::WriteError> for HandleError {
    fn from(err: buffer::WriteError) -> Self {
        log::error!("failed to write socket: {err}");
        Self
    }
}

impl From<WlError> for HandleError {
    fn from(err: WlError) -> Self {
        log::error!("failed to handle request: {err}");
        Self
    }
}

pub struct FatalError;

impl<E: std::fmt::Display> From<E> for FatalError {
    fn from(value: E) -> Self {
        log::error!("{value}");
        Self
    }
}
