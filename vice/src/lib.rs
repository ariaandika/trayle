use std::{collections::HashMap, os::unix::net::UnixStream, path::Path, sync::Arc};
use anyhow::{Context, Result};
use smithay::{
    backend::{
        allocator::{
            gbm::{GbmAllocator, GbmBufferFlags, GbmDevice},
            Fourcc,
        },
        drm::{
            compositor::FrameFlags,
            output::{DrmOutput, DrmOutputManager, DrmOutputRenderElements},
            DrmDevice, DrmDeviceFd, DrmNode, NodeType,
        },
        egl::{context::ContextPriority, EGLDevice, EGLDisplay},
        input::InputEvent,
        libinput::{LibinputInputBackend, LibinputSessionInterface},
        renderer::{
            element as element_utils,
            gles::GlesRenderer,
            multigpu::{gbm::GbmGlesBackend, GpuManager, MultiRenderer},
            utils as renderer_utils, Color32F, ImportAll, ImportEgl, ImportMem,
        },
        session::{self, libseat::LibSeatSession, Session},
        udev::{self, UdevBackend},
    },
    desktop::{
        self, space::SpaceRenderElements, utils::OutputPresentationFeedback, PopupKind,
        PopupManager, Space, Window,
    },
    input::{SeatHandler, SeatState},
    output::{Output, PhysicalProperties},
    reexports::{
        calloop::{
            self,
            generic::{Generic, NoIoDrop},
            EventLoop, Interest, LoopHandle, LoopSignal, Readiness, RegistrationToken,
        },
        drm::control::{connector, crtc, Device as ControlDevice, ModeTypeFlags},
        input::Libinput,
        rustix::fs::OFlags,
        wayland_server::{
            backend::{ClientData, GlobalId},
            protocol::{wl_buffer::WlBuffer, wl_seat::WlSeat, wl_surface::WlSurface},
            Client, Display, DisplayHandle,
        },
    },
    utils::{Clock, Monotonic, Serial, Time},
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



type Gpus = GpuManager<GbmGlesBackend<GlesRenderer, DrmDeviceFd>>;

type UdevRenderer<'a> = MultiRenderer<
    'a,
    'a,
    GbmGlesBackend<GlesRenderer, DrmDeviceFd>,
    GbmGlesBackend<GlesRenderer, DrmDeviceFd>,
>;

#[allow(dead_code)]
pub struct Vice {
    lh: LoopHandle<'static, Vice>,
    dh: DisplayHandle,
    signal: LoopSignal,
    space: Space<Window>,
    popups: PopupManager,
    clock: Clock<Monotonic>,
    socket_name: String,
    primary_gpu: DrmNode,

    gpus: Gpus,
    devices: HashMap<DrmNode, Device>,

    compositor_state: CompositorState,
    xdg_shell_state: XdgShellState,
    seat_state: SeatState<Vice>,
    shm_state: ShmState,
    output_manager_state: OutputManagerState,
    data_device_state: DataDeviceState,
}

impl Vice {
    pub fn setup(event_loop: &mut EventLoop<'static,Vice>) -> Result<Self> {
        let display = Display::<Vice>::new()?;
        let mut space = Space::<Window>::default();
        let lh = event_loop.handle();
        let dh = display.handle();

        // Backend

        let (mut session, session_source) = LibSeatSession::new()?;
        let seat_name = session.seat();

        let mut libinput = Libinput::new_with_udev::<LibinputSessionInterface<_>>(session.clone().into());
        libinput.udev_assign_seat(&seat_name).unwrap();
        let input_source = LibinputInputBackend::new(libinput.clone());

        let primary_gpu = match std::env::var("VICE_DRM_DEVICE") {
            Ok(var) => DrmNode::from_path(var)?,
            _ => match udev::primary_gpu(&seat_name)?
                    .and_then(|x| DrmNode::from_path(x).ok()?.node_with_type(NodeType::Render)?.ok())
            {
                Some(primary_gpu) => primary_gpu,
                None => udev::all_gpus(&seat_name)?
                    .into_iter()
                    .find_map(|x| DrmNode::from_path(x).ok())
                    .context("no GPU available")?,
            },
        };
        tracing::info!("using `{}` as primary gpu.", primary_gpu);

        let api = GbmGlesBackend::with_context_priority(ContextPriority::High);
        let mut gpus: Gpus = GpuManager::new(api)?;

        // the primary gpu is not listed here
        // but it will appear in EGLDevice::try_get_render_node()
        let mut devices = HashMap::new();
        let udev_source = UdevBackend::new(&seat_name)?;
        for (device_id, path) in udev_source.device_list() {
            let node = DrmNode::from_dev_id(device_id)?;
            let device = device::gpus_add_node(node, path, &dh, &lh, &mut session, &mut gpus, &mut space)?;
            devices.insert(node, device);
        }

        let mut renderer = gpus.single_renderer(&primary_gpu)?;
        match ImportEgl::bind_wl_display(&mut renderer, &dh) {
            Ok(_) => tracing::info!("EGL hardware-acceleration enabled"),
            Err(err) => tracing::info!("EGL hardware-acceleration disabled: {err}"),
        }

        // Frontend

        let popups = Default::default();
        let clock = Clock::new();
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

        let display_source = Generic::new(display, Interest::READ, calloop::Mode::Level);

        lh.insert_source(socket_source, handlers::socket).unwrap();
        lh.insert_source(display_source, handlers::display).unwrap();
        lh.insert_source(session_source, handlers::session).unwrap();
        lh.insert_source(input_source, handlers::input).unwrap();
        // lh.insert_source(udev_source, handlers::udev).unwrap();

        tracing::info!("setup finish");

        Ok(Self {
            lh,
            dh,
            signal,
            space,
            popups,
            clock,
            socket_name,
            primary_gpu,

            gpus,
            devices,

            compositor_state,
            xdg_shell_state,
            seat_state,
            shm_state,
            output_manager_state,
            data_device_state,
        })
    }

    pub fn refresh(&mut self) {
        self.space.refresh();
        self.popups.cleanup();
        self.dh.flush_clients().unwrap();
    }

    pub fn socket_name(&self) -> &str {
        &self.socket_name
    }
}

#[derive(Default)]
struct ClientState {
    compositor_state: CompositorClientState,
}

impl ClientData for ClientState { }


/// a single [`Device`] is tagged by [`DrmNode`]
///
/// a single [`Device`] could have multiple [`Surface`]
///
/// a single [`Surface`] is tagged by [`crtc::Handle`]
#[allow(dead_code)]
struct Device {
    device_token: RegistrationToken,
    render_node: DrmNode,
    drm_output_manager: DrmOutputManager<
        GbmAllocator<DrmDeviceFd>,
        GbmDevice<DrmDeviceFd>,
        Option<OutputPresentationFeedback>,
        DrmDeviceFd,
    >,
    surfaces: HashMap<crtc::Handle, Surface>
}

/// a single [`Surface`] is tagged by [`crtc::Handle`]
#[allow(dead_code)]
struct Surface {
    global_output: GlobalId,
    render_node: DrmNode,
    drm_output: DrmOutput<
        GbmAllocator<DrmDeviceFd>,
        GbmDevice<DrmDeviceFd>,
        Option<OutputPresentationFeedback>,
        DrmDeviceFd,
    >,
}

/// a single [`Output`] is tagged by [`UdevOutputId`]
///
/// via provided [`Output`], one can get a [`Surface`] in a [`Device`]
///
/// used by [`render`]
#[derive(PartialEq)]
struct UdevOutputId {
    node: DrmNode,
    crtc: crtc::Handle,
}

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
        unsafe { display.get_mut() }.dispatch_clients(vice).unwrap();
        Ok(calloop::PostAction::Continue)
    }

    pub fn session(event: session::Event, _: &mut (), _: &mut Vice) {
        tracing::warn!("session event {event:?} is not yet implemented");
    }

    pub fn input(event: InputEvent<LibinputInputBackend>, _: &mut (), vice: &mut Vice) {
        match event {
            InputEvent::Keyboard { event } => {
                tracing::debug!(?event);
                vice.signal.stop();
            }
            _ => {}
        }
    }

    #[allow(unused)]
    pub fn udev(event: udev::UdevEvent, _: &mut (), _: &mut Vice) {
        macro_rules! node {
            ($dev_id:tt) => {
                match DrmNode::from_dev_id($dev_id) {
                    Ok(ok) => ok,
                    Err(err) => {
                        tracing::error!("invalid device id from udev: {err}");
                        return;
                    }
                }
            };
        }

        match event {
            udev::UdevEvent::Added { device_id, path } => {
                let node = node!(device_id);
                tracing::info!("udev added device {node} ({path:?})");
            }
            udev::UdevEvent::Changed { device_id } => {
                let node = node!(device_id);
                tracing::info!("udev changed device {node}");
            }
            udev::UdevEvent::Removed { device_id } => {
                let node = node!(device_id);
                tracing::info!("udev removed device {node}");
            }
        }
    }
}

mod device {
    //! single device could have multiple pair of crtc and surface

    use smithay::backend::renderer::element::surface::WaylandSurfaceRenderElement;

    use crate::render::OutputRenderElements;

    use super::*;

    pub fn gpus_add_node(
        node: DrmNode,
        path: &Path,
        dh: &DisplayHandle,
        lh: &LoopHandle<'static, Vice>,
        session: &mut LibSeatSession,
        gpus: &mut Gpus,
        space: &mut Space<Window>,
    ) -> Result<Device> {
        // let node = DrmNode::from_dev_id(device_id)?;
        let fd = session.open(&path, OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOCTTY | OFlags::NONBLOCK)?;
        let fd = DrmDeviceFd::new(fd.into());

        let (device, drm_source) = DrmDevice::new(fd.clone(), true)?;
        let gbm = GbmDevice::new(fd)?;

        // NOTE: #1 add render node to gpus
        let render_node = {
            // EGLDevice::try_get_render_node() will return primary gpu,
            // instead of current node which listed from Udev::device_list()
            let egl_device = EGLDevice::device_for_display(&unsafe { EGLDisplay::new(gbm.clone()) }?)?;
            let render_node = egl_device.try_get_render_node().ok().flatten().unwrap_or(node);
            gpus.as_mut().add_node(render_node, gbm.clone())?;
            render_node
        };

        // NOTE: #2 setup VBlank event listener
        let device_token = lh.insert_source(drm_source, |_,_,_|{
            tracing::trace!("VBlank event");
        }).unwrap();

        let mut renderer = gpus.single_renderer(&render_node)?;

        // NOTE: #3 setup DrmOutputManager
        let mut drm_output_manager = {
            let allocator = GbmAllocator::new(gbm.clone(), GbmBufferFlags::RENDERING | GbmBufferFlags::SCANOUT);
            let color_formats = [Fourcc::Abgr2101010, Fourcc::Argb2101010, Fourcc::Abgr8888, Fourcc::Argb8888];
            let renderer_formats = renderer.as_mut().egl_context().dmabuf_render_formats().clone();
            DrmOutputManager::<_,_,Option<OutputPresentationFeedback>,_>::new(
                device, allocator, gbm.clone(), Some(gbm), color_formats, renderer_formats
            )
        };

        let device = drm_output_manager.device();

        // following setup usually also called on udev device change event

        // NOTE: #4 setup crtc connector stuff
        // FIXME: ideally, single device could have multiple pair of crtc and surface
        let (connector,crtc) = {
            let resource = device.resource_handles()?;
            resource
                .connectors()
                .iter()
                .find_map(|conn|{
                    // TODO: maybe add error handling, or logging instead of silently skipping
                    let connector = device.get_connector(*conn, true).ok()?;
                    match connector.state() {
                        connector::State::Connected => {}
                        connector::State::Disconnected |
                            connector::State::Unknown => {
                                tracing::warn!("skipping connector {connector}");
                                return None;
                            },
                    }

                    let crtc = connector.encoders()
                        .iter()
                        .flat_map(|enc|device.get_encoder(*enc))
                        .find_map(|encoder|{
                            resource
                                .filter_crtcs(encoder.possible_crtcs())
                                .first()
                                .copied()
                        })?;

                    Some((connector,crtc))
                })
                .context("no available pair of connector and crtc")?
        };


        // NOTE: #5 setup physical properties for Output
        let physical = {
            let props = device.get_properties(connector.handle())?;
            let (info,value) = props.into_iter()
                .find_map(|(handle, value)| {
                    let info = device.get_property(handle).ok()?;
                    matches!(info.name().to_str(),Ok("EDID")).then_some(())?;
                    Some((info, value))
                })
                .context("cannot get EDID value from device")?;
            let blob = info.value_type().convert_value(value).as_blob().context("cannot get blob value")?;
            let data = device.get_property_blob(blob)?;
            let edid = libdisplay_info::info::Info::parse_edid(&data)?;

            let make = edid.make().unwrap_or_else(||"Unknown".into());
            let model = edid.model().unwrap_or_else(||"Unknown".into());
            let size = connector.size().map(|(w,h)|(w as i32,h as i32)).unwrap_or((0,0)).into();
            let subpixel = connector.subpixel().into();
            PhysicalProperties { size, subpixel, make, model }
        };

        let mode = match connector.modes().iter()
            .find(|mode| mode.mode_type().contains(ModeTypeFlags::PREFERRED))
        {
            Some(mode) => *mode,
            None => *connector.modes().first().context("no mode available in connector")?,
        };

        // NOTE: #6 setup Output
        let output = {
            let output_name = format!("{}-{}",connector.interface().as_str(),connector.interface_id());
            let wl_mode = mode.into();

            let output = Output::new(output_name, physical);
            let pos = (0,0).into();

            output.change_current_state(Some(wl_mode), None, None, Some(pos));
            output.set_preferred(wl_mode);
            output.user_data().get_or_insert(||UdevOutputId{node,crtc});
            space.map_output(&output, pos);

            // TODO: output user data
            // let damage_tracker = OutputDamageTracker::from_output(&output);

            output
        };

        let global_output = output.create_global::<Vice>(dh);

        {
            let resource = device.resource_handles()?;
            let connectors = resource.connectors();
            let crtcs = resource.connectors();
            tracing::info!(?connectors,?crtcs,"setup ok {path:?}");
        }

        // NOTE: #7 setup DrmOutput
        let drm_output = {
            let planes = device.planes(&crtc)?;
            drm_output_manager
                .initialize_output::<_, OutputRenderElements<UdevRenderer<'_>, WaylandSurfaceRenderElement<UdevRenderer<'_>>>>(
                    crtc,
                    mode,
                    &[connector.handle()],
                    &output,
                    Some(planes),
                    &mut renderer,
                    &DrmOutputRenderElements::default(),
                )?
        };

        let device = {
            let mut surfaces = HashMap::with_capacity(1);
            surfaces.insert(crtc, Surface {
                global_output,
                render_node,
                drm_output,
            });
            Device {
                device_token,
                render_node,
                drm_output_manager,
                surfaces,
            }
        };

        lh.insert_idle(move|vice|{
            if let Err(err) = render::surface(
                node,
                crtc,
                vice.clock.now(),
                vice.primary_gpu,
                &mut vice.space,
                &mut vice.devices,
                &mut vice.gpus,
            ) {
                tracing::error!("render surface error: {err}");
            };
        });

        Ok(device)
    }
}

mod render {
    use super::*;

    smithay::render_elements! {
        /// the final [`RenderElement`] passed to [`desktop::space::space_render_elements`]
        ///
        /// because it only accept a single [`RenderElement`], the macro help to create an enum
        /// to support multiple [`RenderElement`]s
        ///
        /// [`RenderElement`]: smithay::backend::renderer::element
        pub OutputRenderElements<R, E> where R: ImportAll + ImportMem;

        /// elements from [`Space`], which presumably windows
        Space=SpaceRenderElements<R, E>,
    }

    // FIXME: a custom window render elements used internally for [`Space`]
    // smithay::render_elements! {
    //     pub WindowRenderElements<R> where R: ImportAll + ImportMem;
    // }

    /// render for single crtc and surface
    pub fn surface(
        node: DrmNode,
        crtc: crtc::Handle,
        _frame_target: Time<Monotonic>,
        primary_gpu: DrmNode,
        space: &mut Space<Window>,
        devices: &mut HashMap<DrmNode, Device>,
        gpus: &mut Gpus,
    ) -> Result<()> {
        let output = space.outputs()
            .find(|o|{
                o.user_data().get::<UdevOutputId>()==Some(&UdevOutputId{node,crtc})
            })
            .cloned()
            .context("no output matching provided node and crtc")?;

        let device = devices.get_mut(&node).context("invalid node")?;
        let surface = device.surfaces.get_mut(&crtc).context("invalid crtc")?;
        let render_node = surface.render_node;

        let mut renderer = if primary_gpu == render_node {
            gpus.single_renderer(&render_node)
        } else {
            let format = surface.drm_output.format();
            gpus.renderer(&primary_gpu, &render_node, format)
        }?;

        let _res = render_surface(surface, &mut renderer, space, &output)?;

        // do something post render

        Ok(())
    }

    fn render_surface<'a>(
        surface: &'a mut Surface,
        renderer: &mut UdevRenderer<'a>,
        space: &Space<Window>,
        output: &Output,
    ) -> Result<(bool, element_utils::RenderElementStates)> {
        let clear_color = Color32F::new(0.1, 0.1, 0.9, 1.0);
        let frame_mode = FrameFlags::DEFAULT;

        // get all elements

        let elements = desktop::space::space_render_elements(renderer, [space], output, 1.0)?
            .into_iter()
            .map(OutputRenderElements::Space)
            .collect::<Vec<_>>();

        // NOTE: showtime
        // FIXME: gracefull error handling that can reschedule rendering
        let (rendered,render_states) = {
            let frame = surface.drm_output.render_frame(renderer, &elements, clear_color, frame_mode)?;
            (!frame.is_empty,frame.states)
        };

        // NOTE: update_primary_scanout_output
        {
            space.elements().for_each(|window| {
                window.with_surfaces(|surface, states| {
                    desktop::utils::update_surface_primary_scanout_output(
                        surface,
                        output,
                        states,
                        &render_states,
                        element_utils::default_primary_scanout_output_compare,
                    );
                });
            });
            let map = smithay::desktop::layer_map_for_output(output);
            for layer_surface in map.layers() {
                layer_surface.with_surfaces(|surface, states| {
                    desktop::utils::update_surface_primary_scanout_output(
                        surface,
                        output,
                        states,
                        &render_states,
                        element_utils::default_primary_scanout_output_compare,
                    );
                });
            }
        }

        if rendered {
            let mut output_presentation_feedback = OutputPresentationFeedback::new(output);
            space.elements().for_each(|window| {
                if space.outputs_for_element(window).contains(output) {
                    window.take_presentation_feedback(
                        &mut output_presentation_feedback,
                        desktop::utils::surface_primary_scanout_output,
                        |surface, _| {
                            desktop::utils::surface_presentation_feedback_flags_from_states(surface, &render_states)
                        },
                    );
                }
            });
            let map = smithay::desktop::layer_map_for_output(output);
            for layer_surface in map.layers() {
                layer_surface.take_presentation_feedback(
                    &mut output_presentation_feedback,
                    desktop::utils::surface_primary_scanout_output,
                    |surface, _| {
                        desktop::utils::surface_presentation_feedback_flags_from_states(surface, &render_states)
                    },
                );
            }
            // NOTE: take_presentation_feedback
            surface
                .drm_output
                .queue_frame(Some(output_presentation_feedback))?;
        }

        Ok((rendered,render_states))
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

