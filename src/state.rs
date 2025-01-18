use std::{ffi::OsString, sync::Arc, time::Instant};

use smithay::{
    desktop::{PopupManager, Space, Window, WindowSurfaceType},
    input::{Seat, SeatState},
    reexports::{
        calloop::{generic::Generic, EventLoop, Interest, LoopSignal, Mode, PostAction},
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason},
            protocol::wl_surface::WlSurface,
            Display, DisplayHandle,
        },
    },
    utils::{Logical, Point},
    wayland::{
        compositor::{CompositorClientState, CompositorState},
        dmabuf::DmabufState,
        output::OutputManagerState,
        selection::data_device::DataDeviceState,
        shell::xdg::XdgShellState,
        shm::ShmState,
        socket::ListeningSocketSource,
    },
};

const KB_REPEAT_DELAY: i32 = 160;
const KB_REPEAT_RATE: i32 = 50;

pub trait BackendState {
    fn state(&self) -> &State;
    fn state_mut(&mut self) -> &mut State;
}

pub struct State {
    pub start_time: Instant,
    pub socket_name: OsString,
    pub display_handle: DisplayHandle,

    pub space: Space<Window>,
    pub loop_signal: LoopSignal,

    // Our Own State
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub output_manager_state: OutputManagerState,
    pub seat_state: SeatState<State>,
    pub data_device_state: DataDeviceState,
    pub popups: PopupManager,

    pub seat: Seat<State>,
    pub dmabuf_state: DmabufState,
}

impl State {
    pub fn new<Backend>(event_loop: &mut EventLoop<Backend>, display: Display<Self>) -> Self
    where
        Backend: BackendState
    {
        let start_time = Instant::now();

        let dh = display.handle();

        let compositor_state = CompositorState::new::<Self>(&dh);
        let xdg_shell_state = XdgShellState::new::<Self>(&dh);
        let shm_state = ShmState::new::<Self>(&dh, vec![]);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&dh);
        let mut seat_state = SeatState::new();
        let data_device_state = DataDeviceState::new::<Self>(&dh);
        let popups = PopupManager::default();
        let dmabuf_state = DmabufState::new();

        // NOTE: A seat is a group of keyboards, pointer and touch devices.
        // A seat typically has a pointer and maintains a keyboard focus and a pointer focus.
        // TODO: for now, assuming device is always available
        //  the future, one may track keyboard hot-plug in
        let mut seat: Seat<Self> = seat_state.new_wl_seat(&dh, std::env!("CARGO_CRATE_NAME"));
        seat.add_keyboard(Default::default(), KB_REPEAT_DELAY, KB_REPEAT_RATE).unwrap();
        seat.add_pointer();


        // NOTE: A space represents a two-dimensional plane. Windows and Outputs can be mapped onto it.
        // Windows get a position and stacking order through mapping.
        // Outputs become views of a part of the Space and can be rendered via Space::render_output.
        let space = Space::default();

        let socket_name = Self::init_wayland_listener(display, event_loop);

        // NOTE: Get the loop signal, used to stop the event loop
        let loop_signal = event_loop.get_signal();

        Self {
            start_time,
            display_handle: dh,

            space,
            loop_signal,
            socket_name,

            compositor_state,
            xdg_shell_state,
            shm_state,
            output_manager_state,
            seat_state,
            data_device_state,
            popups,
            seat,
            dmabuf_state,
        }
    }

    fn init_wayland_listener<Backend>(
        display: Display<State>,
        event_loop: &mut EventLoop<'_, Backend>,
    ) -> OsString where Backend: BackendState {
        // NOTE: Creates a new listening socket, automatically choosing the next available `wayland` socket name.
        let wl_socket = ListeningSocketSource::new_auto().unwrap();
        let socket_name = wl_socket.socket_name().to_os_string();

        let loop_handle = event_loop.handle();

        // NOTE: `insert_resource` insert new **EventSource**
        // in this case, event when client connected
        loop_handle
            .insert_source(wl_socket, move |client_stream, _, backend|{
                backend
                    .state_mut()
                    .display_handle
                    .insert_client(client_stream, Arc::new(ClientState::default()))
                    .unwrap();
            })
            .expect("Failed to init wayland event source");

        // NOTE: add display to the event loop,
        // so that client events will be processed by wayland-server.
        loop_handle
            .insert_source(
                Generic::new(display, Interest::READ, Mode::Level),
                |_, display, backend| {
                    // Safety: we dont drop display
                    unsafe {
                        display.get_mut().dispatch_clients(backend.state_mut()).unwrap();
                    }
                    Ok(PostAction::Continue)
                }
            )
            .unwrap();

        socket_name
    }

    pub fn surface_under(&self, pos: Point<f64, Logical>) -> Option<(WlSurface, Point<f64, Logical>)> {
        self.space.element_under(pos).and_then(|(window, location)|{
            window
                .surface_under(pos - location.to_f64(), WindowSurfaceType::ALL)
                .map(|(s, p)| (s, (p + location).to_f64()))
        })
    }
}

/// for testing purposes
impl State {
    pub fn open_terminal(&self) {
        use std::process::Command;

        match Command::new("alacritty").spawn() {
            Ok(_child) => {},
            Err(err) => {
                eprintln!("Alacritty spawn error: {err:?}");
            },
        }
    }
}

/// each client state
#[derive(Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) { }
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) { }
}

