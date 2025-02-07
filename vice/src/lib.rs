use std::{os::unix::net::UnixStream, sync::Arc};
use anyhow::Result;
use smithay::{
    backend::{
        input::InputEvent,
        libinput::{LibinputInputBackend, LibinputSessionInterface},
        renderer::{damage::OutputDamageTracker, utils as renderer_utils},
        session::{self, libseat::LibSeatSession, Session},
    },
    desktop::{self, PopupKind, PopupManager, Space, Window},
    input::{SeatHandler, SeatState},
    output::{self, Output, PhysicalProperties, Subpixel},
    reexports::{
        calloop::{
            self,
            generic::{Generic, NoIoDrop},
            EventLoop, Interest, LoopHandle, LoopSignal, Readiness,
        },
        input::Libinput,
        wayland_server::{
            backend::ClientData,
            protocol::{
                wl_buffer::WlBuffer, wl_seat::WlSeat, wl_surface::WlSurface,
            },
            Client, Display, DisplayHandle,
        },
    },
    utils::Serial,
    wayland::{
        buffer::BufferHandler,
        compositor::{self, CompositorClientState, CompositorHandler, CompositorState},
        output::{OutputHandler, OutputManagerState},
        selection::{
            data_device::{
                ClientDndGrabHandler, DataDeviceHandler, DataDeviceState, ServerDndGrabHandler,
            },
            SelectionHandler,
        },
        shell::xdg::{
            PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
            XdgToplevelSurfaceData,
        },
        shm::{ShmHandler, ShmState},
        socket::ListeningSocketSource,
    },
};




#[allow(dead_code)]
pub struct Vice {
    lh: LoopHandle<'static, Vice>,
    dh: DisplayHandle,
    signal: LoopSignal,
    space: Space<Window>,
    popups: PopupManager,
    socket_name: String,
    damage_tracker: OutputDamageTracker,

    compositor_state: CompositorState,
    xdg_shell_state: XdgShellState,
    seat_state: SeatState<Vice>,
    shm_state: ShmState,
    output_manager_state: OutputManagerState,
    data_device_state: DataDeviceState,
}

impl Vice {
    pub fn setup(event_loop: &mut EventLoop<'static,Vice>) -> Result<Self> {
        // Backend

        let (session, session_source) = LibSeatSession::new()?;
        let seat_name = session.seat();
        let mut libinput = Libinput::new_with_udev::<LibinputSessionInterface<_>>(session.clone().into());
        libinput.udev_assign_seat(&seat_name).unwrap();
        let input_source = LibinputInputBackend::new(libinput.clone());

        // Frontend

        let display = Display::<Vice>::new()?;
        let mut space = Space::<Window>::default();
        let popups = Default::default();

        let lh = event_loop.handle();
        let dh = display.handle();
        let signal = event_loop.get_signal();

        let compositor_state = CompositorState::new::<Vice>(&dh);
        let xdg_shell_state = XdgShellState::new::<Vice>(&dh);
        let mut seat_state = SeatState::<Vice>::new();
        let shm_state = ShmState::new::<Vice>(&dh, []);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Vice>(&dh);
        let data_device_state = DataDeviceState::new::<Vice>(&dh);

        let mut seat = seat_state.new_wl_seat(&dh, &seat_name);
        seat.add_keyboard(Default::default(), 150, 50).unwrap();

        let socket_source = ListeningSocketSource::new_auto()?;
        let socket_name = socket_source.socket_name().to_string_lossy().into_owned();

        let mode = output::Mode {
            size: (640,480).into(),
            refresh: 60_000,
        };

        let output = Output::new(
            "deez".into(),
            PhysicalProperties {
                size: (0, 0).into(),
                subpixel: Subpixel::Unknown,
                make: "pretzel".into(),
                model: "Deez".into(),
            },
        );
        let _global = output.create_global::<Vice>(&dh);

        output.change_current_state(Some(mode), None, None, Some((0,0).into()));
        output.set_preferred(mode);
        space.map_output(&output, (0,0));

        let damage_tracker = OutputDamageTracker::from_output(&output);
        let display_source = Generic::new(display, Interest::READ, calloop::Mode::Level);

        lh.insert_source(session_source, handlers::session).unwrap();
        lh.insert_source(input_source, handlers::input).unwrap();

        lh.insert_source(socket_source, handlers::socket).unwrap();
        lh.insert_source(display_source, handlers::display).unwrap();

        Ok(Self {
            lh,
            dh,
            signal,
            space,
            popups,
            socket_name,
            damage_tracker,
            compositor_state,
            xdg_shell_state,
            seat_state,
            shm_state,
            output_manager_state,
            data_device_state,
        })
    }
}

#[derive(Default)]
struct ClientState {
    compositor_state: CompositorClientState,
}

impl ClientData for ClientState { }


mod xdg_shell {
    use super::*;

    pub fn handle_commit(popups: &mut PopupManager, space: &Space<Window>, surface: &WlSurface) {
        if let Some(window) = space
            .elements()
            .find(|w| w.toplevel().unwrap().wl_surface() == surface)
            .cloned()
        {
            let initial_configure_sent = compositor::with_states(surface, |states| {
                states
                    .data_map
                    .get::<XdgToplevelSurfaceData>()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .initial_configure_sent
            });

            if !initial_configure_sent {
                window.toplevel().unwrap().send_configure();
            }
        }

        popups.commit(surface);

        if let Some(popup) = popups.find_popup(surface) {
            match popup {
                PopupKind::Xdg(ref xdg) => {
                    if !xdg.is_initial_configure_sent() {
                        xdg.send_configure().expect("should never fail");
                    }
                }
                PopupKind::InputMethod(_) => {}
            }
        }
    }
}

mod util {
    use super::*;

    pub fn unconstraint_popup(popup: &PopupSurface, vice: &mut Vice) {
        let Ok(root) = desktop::find_popup_root_surface(&PopupKind::Xdg(popup.clone())) else {
            return;
        };

        let Some(window) = vice.space.elements().find(|w|w.toplevel().unwrap().wl_surface() == &root) else {
            return;
        };

        let output = vice.space.outputs().next().unwrap();
        let output_geo = vice.space.output_geometry(output).unwrap();
        let window_geo = vice.space.element_geometry(window).unwrap();

        let mut target = output_geo;
        target.loc -= desktop::get_popup_toplevel_coords(&PopupKind::Xdg(popup.clone()));
        target.loc -= window_geo.loc;

        popup.with_pending_state(|state|{
            state.geometry = state.positioner.get_unconstrained_geometry(target);
        });
    }
}

mod handlers {
    use super::*;

    pub fn socket(stream: UnixStream, _: &mut (), vice: &mut Vice) {
        vice.dh.insert_client(stream, Arc::new(ClientState::default())).unwrap();
    }

    pub fn display(_: Readiness, display: &mut NoIoDrop<Display<Vice>>, vice: &mut Vice) -> std::io::Result<calloop::PostAction> {
        unsafe {
            display.get_mut().dispatch_clients(vice).unwrap();
        }
        Ok(calloop::PostAction::Continue)
    }

    pub fn input(event: InputEvent<LibinputInputBackend>, _: &mut (), vice: &mut Vice) {
        match event {
            InputEvent::Keyboard { event } => {
                dbg!(event);
                vice.signal.stop();
            }
            _ => {}
        }
    }

    pub fn session(_: session::Event, _: &mut (), _: &mut Vice) {
    }
}

smithay::delegate_compositor!(Vice);

impl CompositorHandler for Vice {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        renderer_utils::on_commit_buffer_handler::<Vice>(surface);
        if !compositor::is_sync_subsurface(surface) {
            let mut root = surface.clone();
            while let Some(parent) = compositor::get_parent(&root) {
                root = parent;
            }
            let toplevel_window = self.space.elements().find(|w|w.toplevel().unwrap().wl_surface() == &root);
            if let Some(window) = toplevel_window {
                window.on_commit();
            }
        }

        xdg_shell::handle_commit(&mut self.popups, &self.space, surface);
        // todo!("resize_grab::handle_commit")
    }
}

smithay::delegate_xdg_shell!(Vice);

impl XdgShellHandler for Vice {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let window = Window::new_wayland_window(surface);
        self.space.map_element(window, (0,0), false);
    }

    fn new_popup(&mut self, surface: PopupSurface, _positioner: PositionerState) {
        util::unconstraint_popup(&surface, self);
        let _ = self.popups.track_popup(PopupKind::Xdg(surface));
    }

    fn grab(&mut self, _surface: PopupSurface, _seat: WlSeat, _serial: Serial) {
        // todo!("popup grab")
    }

    fn reposition_request(&mut self, surface: PopupSurface, positioner: PositionerState, token: u32) {
        surface.with_pending_state(|state|{
            let geometry = positioner.get_geometry();
            state.geometry = geometry;
            state.positioner = positioner;
        });
        util::unconstraint_popup(&surface, self);
        surface.send_repositioned(token);
    }
}

smithay::delegate_seat!(Vice);

impl SeatHandler for Vice {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut smithay::input::SeatState<Self> {
        &mut self.seat_state
    }
}

smithay::delegate_shm!(Vice);

impl ShmHandler for Vice {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl BufferHandler for Vice {
    fn buffer_destroyed(&mut self, _buffer: &WlBuffer) { }
}

smithay::delegate_output!(Vice);

impl OutputHandler for Vice { }

smithay::delegate_data_device!(Vice);

impl DataDeviceHandler for Vice {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}

impl SelectionHandler for Vice {
    type SelectionUserData = ();
}

impl ClientDndGrabHandler for Vice { }
impl ServerDndGrabHandler for Vice { }

