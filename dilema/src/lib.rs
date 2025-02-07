//! an attempt for more distributed project structure
use anyhow::{anyhow, bail, Context, Result};
use drm_scanner::DrmScanner;
use smithay::{
    backend::{
        allocator::{
            dmabuf::Dmabuf,
            format::FormatSet,
            gbm::{GbmAllocator, GbmBufferFlags, GbmDevice},
            Fourcc,
        },
        drm::{
            output::{DrmOutput, DrmOutputManager, DrmOutputRenderElements},
            DrmDevice, DrmDeviceFd, DrmEvent, DrmEventMetadata, DrmNode, DrmSurface, NodeType,
        },
        egl::{context::ContextPriority, EGLDevice, EGLDisplay},
        input::{Event as _, InputEvent, KeyboardKeyEvent},
        libinput::{LibinputInputBackend, LibinputSessionInterface},
        renderer::{
            element::solid::SolidColorRenderElement,
            gles::GlesRenderer,
            multigpu::{gbm::GbmGlesBackend, GpuManager, MultiRenderer},
            utils as renderer_utils, Color32F, DebugFlags, ImportDma, ImportEgl, ImportMemWl,
        },
        session::{self, libseat::LibSeatSession, Session},
        udev::{self, UdevBackend, UdevEvent},
    },
    desktop::{utils::OutputPresentationFeedback, Space, Window},
    input::{
        keyboard::{FilterResult, Keysym, KeysymHandle, ModifiersState, XkbConfig},
        Seat, SeatHandler, SeatState,
    },
    output::{Output, PhysicalProperties},
    reexports::{
        calloop::{
            self,
            generic::{Generic, NoIoDrop},
            EventLoop, Interest, LoopHandle, LoopSignal, PostAction, Readiness, RegistrationToken,
        },
        drm::{
            control::{connector, crtc, Device as ControlDevice, ModeTypeFlags},
            Device as _,
        },
        input::{DeviceCapability, Libinput},
        rustix::fs::OFlags,
        wayland_protocols::wp::linux_dmabuf::zv1::server::zwp_linux_dmabuf_feedback_v1,
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason, GlobalId},
            protocol::{
                wl_buffer::WlBuffer, wl_output::WlOutput, wl_seat::WlSeat, wl_surface::WlSurface,
            },
            Client, Display, DisplayHandle,
        },
    },
    utils::{Clock, Monotonic, Serial, SERIAL_COUNTER},
    wayland::{
        buffer::BufferHandler,
        compositor::{CompositorClientState, CompositorHandler, CompositorState},
        dmabuf::{
            DmabufFeedback, DmabufFeedbackBuilder, DmabufGlobal, DmabufHandler, DmabufState,
            ImportNotifier,
        },
        drm_lease::{
            DrmLease, DrmLeaseBuilder, DrmLeaseHandler, DrmLeaseRequest, DrmLeaseState,
            LeaseRejected,
        },
        drm_syncobj::{self, DrmSyncobjHandler, DrmSyncobjState},
        output::OutputHandler,
        shell::xdg::{
            PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
        },
        shm::{ShmHandler, ShmState},
        socket::ListeningSocketSource,
    },
};


use std::{collections::HashMap, env::var, os::unix::net::UnixStream, path::Path};

use crate::config::Config;

type UdevRenderer<'a> = MultiRenderer<
    'a,'a,
    GbmGlesBackend<GlesRenderer, DrmDeviceFd>,
    GbmGlesBackend<GlesRenderer, DrmDeviceFd>,
>;

type Gpu = GpuManager<GbmGlesBackend<GlesRenderer, DrmDeviceFd>>;

pub struct Dilema {
    handle: LoopHandle<'static, Dilema>,
    signal: LoopSignal,
    dh: DisplayHandle,

    config: Config,
    space: Space<Window>,
    pub session: LibSeatSession,
    gpus: Gpu,
    libinput: Libinput,
    devices: HashMap<DrmNode,DeviceData>,
    seat: Seat<Dilema>,

    clock: Clock<Monotonic>,
    seat_name: String,
    socket_name: String,
    primary_gpu: DrmNode,

    compositor_state: CompositorState,
    xdg_shell_state: XdgShellState,
    seat_state: SeatState<Dilema>,
    shm_state: ShmState,
    dmabuf_state: DmabufState,
    dmabuf_global: DmabufGlobal,
    drm_syncobj: Option<DrmSyncobjState>,
}

impl Dilema {
    pub fn setup(event_loop: &mut EventLoop<'static,Dilema>) -> Result<Dilema> {
        let display = Display::<Dilema>::new()?;

        let handle = event_loop.handle();
        let signal = event_loop.get_signal();
        let dh = display.handle();

        let config = Config::setup()?;
        let (mut session, session_source) = LibSeatSession::new()?;
        let seat_name = session.seat();

        let primary_gpu = match udev::primary_gpu(&seat_name)?
            .and_then(|gpu|DrmNode::from_path(gpu).ok()?.node_with_type(NodeType::Render)?.ok())
        {
            Some(ok) => ok,
            None => udev::all_gpus(&seat_name)?
                .into_iter()
                .find_map(|gpu|DrmNode::from_path(gpu).ok())
                .context("no gpu found")?,
        };

        tracing::info!("using {primary_gpu} as primary gpu");

        let graphics_api = GbmGlesBackend::with_context_priority(ContextPriority::High);
        let mut gpus: Gpu = GpuManager::new(graphics_api)?;

        // ...

        let socket_source = ListeningSocketSource::new_auto()?;
        let socket_name = socket_source.socket_name().to_string_lossy().into_owned();
        tracing::info!("listening on socket {socket_name:?}");

        let compositor_state = CompositorState::new::<Dilema>(&dh);
        let xdg_shell_state = XdgShellState::new::<Dilema>(&dh);
        let mut seat_state = SeatState::<Dilema>::new();
        let mut shm_state = ShmState::new::<Dilema>(&dh, []);
        let dmabuf_state = DmabufState::new();

        let mut seat = seat_state.new_wl_seat(&dh, &seat_name);
        seat.add_keyboard(XkbConfig::default(), 150, 50)?;

        let clock = Clock::new();
        let mut space = Space::default();

        // ...

        let udev = UdevBackend::new(&seat_name)?;
        let mut libinput = Libinput::new_with_udev::<LibinputSessionInterface<LibSeatSession>>(
            session.clone().into()
        );
        libinput.udev_assign_seat(&seat_name).map_err(|()|anyhow!("failed to assign udev seat"))?;
        input::setup(&mut libinput);
        let input_source = LibinputInputBackend::new(libinput.clone());

        let mut devices = HashMap::<DrmNode,DeviceData>::new();
        device::setup(udev, &mut session, &handle, &dh, &primary_gpu, &config, &mut devices, &mut space, &mut gpus);

        let mut renderer = gpus.single_renderer(&primary_gpu)?;

        shm_state.update_formats(ImportMemWl::shm_formats(&renderer));

        match ImportEgl::bind_wl_display(&mut renderer, &dh) {
            Ok(_) => tracing::info!("EGL hardware-acceleration enabled"),
            Err(err) => tracing::info!("EGL hardware-acceleration disabled, {err}"),
        };

        let dmabuf_formats = ImportDma::dmabuf_formats(&renderer);
        let default_feedback = DmabufFeedbackBuilder::new(primary_gpu.dev_id(),dmabuf_formats).build().unwrap();
        let mut dmabuf = DmabufState::new();
        let dmabuf_global = dmabuf.create_global_with_default_feedback::<Dilema>(&dh, &default_feedback);


        for device in devices.values_mut() {
            for surface in device.surfaces.values_mut() {
                if surface.dmabuf_feedback.is_some() {
                    continue;
                }
                if let Some(dmabuf) = surface.drm_output.with_compositor(|compositor|{
                    device::get_surface_dmabuf_feedback(
                        primary_gpu,
                        surface.render_node,
                        &mut gpus,
                        compositor.surface()
                    )
                }) {
                    surface.dmabuf_feedback.replace(dmabuf);
                }
            }
        }

        let drm_syncobj = primary_gpu
            .node_with_type(NodeType::Primary)
            .and_then(Result::ok)
            .and_then(|primary_node|{
                let device = devices.get(&primary_node)?;
                let import_device = device.drm_output_manager.device().device_fd().clone();
                if !drm_syncobj::supports_syncobj_eventfd(&import_device) {
                    return None;
                }
                Some(DrmSyncobjState::new::<Dilema>(&dh, import_device))
            });

        let display_source = Generic::new(display, Interest::READ, calloop::Mode::Level);

        handle.insert_source(socket_source, callbacks::socket).unwrap();
        handle.insert_source(display_source, callbacks::display).unwrap();
        handle.insert_source(input_source, input::handler).unwrap();
        handle.insert_source(session_source, callbacks::session).unwrap();

        tracing::info!("setup finished");

        Ok(Self {
            handle,
            signal,
            dh,
            config,
            space,
            session,
            gpus,
            libinput,
            devices,
            seat,
            clock,
            seat_name,
            socket_name,
            primary_gpu,
            compositor_state,
            xdg_shell_state,
            seat_state,
            shm_state,
            dmabuf_state,
            dmabuf_global,
            drm_syncobj,
        })
    }

    pub fn refresh(&mut self) {
        tracing::debug!("refresh");
        self.space.refresh();
        self.dh.flush_clients().unwrap();
    }
}

struct DeviceData {
    // handles
    token: RegistrationToken,
    render_node: DrmNode,

    // setup on device added
    drm_lease_state: Option<DrmLeaseState>,
    drm_output_manager: DrmOutputManager<
        GbmAllocator<DrmDeviceFd>,
        GbmDevice<DrmDeviceFd>,
        Option<OutputPresentationFeedback>,
        DrmDeviceFd,
    >,

    // setup later
    drm_scanner: DrmScanner,
    surfaces: HashMap<crtc::Handle, SurfaceData>,
    non_desktop_connectors: Vec<(connector::Handle, crtc::Handle)>,
    active_leases: Vec<DrmLease>,
}

struct SurfaceData {
    dh: DisplayHandle,
    node: DrmNode,
    render_node: DrmNode,

    global: GlobalId,
    drm_output: DrmOutput<
        GbmAllocator<DrmDeviceFd>,
        GbmDevice<DrmDeviceFd>,
        Option<OutputPresentationFeedback>,
        DrmDeviceFd,
    >,
    dmabuf_feedback: Option<SurfaceDmabufFeedback>,

    disable_direct_scanout: bool,
}

#[derive(PartialEq)]
struct UdevOutputId {
    node: DrmNode,
    crtc: crtc::Handle,
}

#[derive(Clone)]
struct SurfaceDmabufFeedback {
    render_feedback: DmabufFeedback,
    scanout_feedback: DmabufFeedback,
}

#[derive(Default)]
struct ClientState {
    compositor_state: CompositorClientState
}

mod config {
    use super::*;

    macro_rules! from_env {
        ($name:tt) => {
            matches!(var($name).as_deref(),Ok("1"))
        };
    }

    pub const SUPPORTED_FORMATS_8BIT_ONLY: &[Fourcc] = &[Fourcc::Abgr8888, Fourcc::Argb8888];
    pub const SUPPORTED_FORMATS: &[Fourcc] = &[
        Fourcc::Abgr2101010,
        Fourcc::Argb2101010,
        Fourcc::Abgr8888,
        Fourcc::Argb8888,
    ];

    pub struct Config {
        pub clear_color: Color32F,
        pub kb_repeat_delay: i32,
        pub kb_repeat_rate: i32,
        pub disable_direct_10bit: bool,
        pub disable_direct_scanout: bool,
    }

    impl Config {
        pub fn setup() -> Result<Config> {
            Ok(Self {
                clear_color: Color32F::new(0.1, 0.1, 0.8, 1.0),
                kb_repeat_delay: 160,
                kb_repeat_rate: 50,
                disable_direct_10bit: false,
                disable_direct_scanout: from_env!("DL_DISABLE_DIRECT_SCANOUT"),
            })
        }
    }
}

mod input {
    use super::*;

    pub fn setup(libinput: &mut Libinput) {
        for event in libinput {
            tracing::debug!("device event {event:?}");
        }
    }

    pub fn handler(event: InputEvent<LibinputInputBackend>, _: &mut (), dilema: &mut Dilema) {
        tracing::debug!("device event {event:?}");

        if let InputEvent::DeviceAdded { device } = event {
            if device.has_capability(DeviceCapability::Keyboard) {
                if let Err(err) = dilema.seat.add_keyboard(
                    Default::default(),
                    dilema.config.kb_repeat_delay,
                    dilema.config.kb_repeat_rate,
                ) {
                    tracing::error!("{err}");
                }
            }
            return;
        }

        if let InputEvent::DeviceRemoved { device } = event {
            if device.has_capability(DeviceCapability::Keyboard) {
                dilema.seat.remove_keyboard();
            }
            return;
        }

        if let InputEvent::Keyboard { event } = event {
            tracing::debug!("keyboard {event:?}");
            let serial = SERIAL_COUNTER.next_serial();
            let time = event.time_msec();
            dilema.seat.get_keyboard().unwrap().input::<(), _>(
                dilema,
                event.key_code(),
                event.state(),
                serial,
                time,
                self::on_keyboard,
            );
            return;
        }
    }

    fn on_keyboard(dilema: &mut Dilema, mods: &ModifiersState, handle: KeysymHandle) -> FilterResult<()> {
        let keysym = handle.modified_sym();
        tracing::debug!(?mods, keysym = ::xkbcommon::xkb::keysym_get_name(keysym), "keysym");

        match keysym {
            Keysym::Return if mods.logo => {
                std::process::Command::new("alacritty")
                    .env("WAYLAND_DISPLAY", &dilema.socket_name)
                    .spawn().inspect_err(|err|tracing::error!("{err}")).ok();
                FilterResult::Intercept(())
            }
            Keysym::Q if mods.logo => {
                tracing::info!("shutting down");
                dilema.signal.stop();
                FilterResult::Intercept(())
            }
            _ => FilterResult::Forward
        }
    }
}

mod callbacks {
    use std::sync::Arc;

    use smithay::wayland::compositor;

    use super::*;

    pub fn socket(stream: UnixStream, _: &mut (), dilema: &mut Dilema) {
        tracing::trace!("callbacks::socket");
        if let Err(err) = dilema
            .dh
            .insert_client(stream, Arc::new(ClientState::default()))
        {
            tracing::error!("{err}");
        }
    }

    pub fn display(
        _: Readiness,
        display: &mut NoIoDrop<Display<Dilema>>,
        dilema: &mut Dilema,
    ) -> std::io::Result<calloop::PostAction> {
        tracing::trace!("callbacks::display");
        unsafe { display.get_mut().dispatch_clients(dilema) }.unwrap();
        Ok(PostAction::Continue)
    }

    pub fn session(event: session::Event, _: &mut (), dilema: &mut Dilema) {
        tracing::trace!("callbacks::session");
        match event {
            session::Event::PauseSession => {
                tracing::info!("session pause");
                dilema.libinput.suspend();
                for device in dilema.devices.values_mut() {
                    device.drm_output_manager.pause();
                    device.active_leases.clear();
                    if let Some(lease) = device.drm_lease_state.as_mut() {
                        lease.suspend();
                    }
                }
            }
            session::Event::ActivateSession => {
                tracing::info!("session resume");
                if let Err(err) = dilema.libinput.resume() {
                    tracing::error!("{err:?}");
                }
                for (node, device) in &mut dilema.devices {
                    // if we do not care about flicking (caused by modesetting) we could just
                    // pass true for disable connectors here. this would make sure our drm
                    // device is in a known state (all connectors and planes disabled).
                    // but for demonstration we choose a more optimistic path by leaving the
                    // state as is and assume it will just work. If this assumption fails
                    // we will try to reset the state when trying to queue a frame.
                    device
                        .drm_output_manager
                        .activate(false)
                        .expect("failed to activate drm backend");
                    if let Some(lease) = device.drm_lease_state.as_mut() {
                        lease.resume::<Dilema>();
                    }
                    let node = *node;
                    dilema.handle
                        .insert_idle(move |dilema| render::node(node, None, dilema.clock.now(), dilema));
                }
            }
        }
    }

    /// called on [`CompositorHandler::commit`]
    pub fn compositor_commit(surface: &WlSurface, dilema: &mut Dilema) {
        renderer_utils::on_commit_buffer_handler::<Dilema>(surface);
        if !compositor::is_sync_subsurface(surface) {
            let mut root = compositor::get_parent(surface).unwrap_or_else(||surface.clone());
            while let Some(parent) = compositor::get_parent(&root) {
                root = parent;
            }
            if let Some(window) = dilema
                .space
                .elements()
                .find(|w| w.toplevel().unwrap().wl_surface() == &root)
            {
                window.on_commit();
            }
        }
    }
}

mod device {
    use super::*;

    struct DeviceProps<'a> {
        handle: &'a LoopHandle<'static, Dilema>,
        dh: &'a DisplayHandle,
        primary_gpu: &'a DrmNode,
        config: &'a Config,
        devices: &'a mut HashMap<DrmNode, DeviceData>,
        space: &'a mut Space<Window>,
        gpus: &'a mut Gpu,
    }

    pub fn setup(
        udev: UdevBackend,
        session: &mut LibSeatSession,
        handle: &LoopHandle<'static, Dilema>,
        dh: &DisplayHandle,
        primary_gpu: &DrmNode,
        config: &Config,
        devices: &mut HashMap<DrmNode, DeviceData>,
        space: &mut Space<Window>,
        gpus: &mut Gpu,
    ) {
        for (device_id,path) in udev.device_list() {
            let result = self::added(device_id, &path, session, DeviceProps {
                handle, dh, primary_gpu, config, devices, space, gpus,
            });

            if let Err(err) = result {
                tracing::error!("{err}");
            }
        }

        handle.insert_source(udev, self::udev_handler).unwrap();
    }

    fn udev_handler(event: UdevEvent, _: &mut (), dilema: &mut Dilema) {
        tracing::trace!("device::udev_handler");

        let props = DeviceProps {
            handle: &dilema.handle,
            dh: &dilema.dh,
            primary_gpu: &dilema.primary_gpu,
            config: &dilema.config,
            devices: &mut dilema.devices,
            space: &mut dilema.space,
            gpus: &mut dilema.gpus,
        };
        let result = match event {
            UdevEvent::Added { device_id, path } => {
                self::added(device_id, &path, &mut dilema.session, props)
            }
            UdevEvent::Changed { device_id } => {
                self::changed(device_id, props)
            }
            UdevEvent::Removed { device_id } => {
                self::removed(device_id, props)
            }
        };

        if let Err(err) = result {
            tracing::error!("{err}");
        }
    }

    fn drm_handler(event: DrmEvent, node: DrmNode, meta: &mut Option<DrmEventMetadata>, dilema: &mut Dilema) {
        tracing::trace!("device::drm_handler");
        match event {
            DrmEvent::VBlank(crtc) => {
                render::frame_finish(node, crtc, meta, dilema);
            }
            DrmEvent::Error(err) => {
                tracing::error!("{err}");
            }
        }
    }

    fn added(device_id: u64, path: &Path, session: &mut LibSeatSession, props: DeviceProps) -> Result<()> {
        let DeviceProps { handle, dh, primary_gpu, config, devices, space, gpus } = props;

        let node = DrmNode::from_dev_id(device_id)?;

        let flags = OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOCTTY | OFlags::NONBLOCK;
        let fd = DrmDeviceFd::new(Session::open(session, path, flags)?.into());
        let (drm,drm_source) = DrmDevice::new(fd.clone(), true)?;
        let gbm = GbmDevice::new(fd)?;

        let display = unsafe { EGLDisplay::new(gbm.clone()) }?;
        let egldevice = EGLDevice::device_for_display(&display)?;
        let render_node = egldevice.try_get_render_node()?.context("deez")?;

        let color_formats = match config.disable_direct_10bit {
            true => config::SUPPORTED_FORMATS_8BIT_ONLY,
            false => config::SUPPORTED_FORMATS,
        };
        let gbm_buffer_flags = GbmBufferFlags::RENDERING | GbmBufferFlags::SCANOUT;
        let allocator = GbmAllocator::new(gbm.clone(), gbm_buffer_flags);

        // NOTE: #1 setup render node
        gpus.as_mut().add_node(render_node, gbm.clone())?;

        let mut renderer = gpus.single_renderer(&render_node).expect("failed to get renderer");
        let render_formats = renderer.as_mut().egl_context().dmabuf_render_formats().clone();

        // setup drm output manager
        let drm_output_manager = DrmOutputManager::new(
            drm,
            allocator,
            gbm.clone(),
            Some(gbm.clone()),
            color_formats.iter().copied(),
            render_formats
        );

        // setup drm lease
        let drm_lease_state = match DrmLeaseState::new::<Dilema>(dh, &node) {
            Ok(ok) => Some(ok),
            Err(err) => {
                tracing::warn!("failed to setup drm lease global for {node}: {err:?}");
                None
            },
        };


        // NOTE: #2 setup vblank event handler
        let token = handle.insert_source(drm_source, move|e,m,d|drm_handler(e,node,m,d)).unwrap();

        // NOTE: #3 setup DeviceData
        let device_data = DeviceData {
            token,
            render_node,

            drm_output_manager,
            drm_lease_state,

            drm_scanner: DrmScanner::new(),
            surfaces: HashMap::new(),
            active_leases: vec![],
            non_desktop_connectors: vec![],
        };
        devices.insert(node, device_data);

        self::changed(device_id, DeviceProps { handle, dh, primary_gpu, config, devices, space, gpus })
    }

    fn connector_connected(
        node: DrmNode,
        connector: connector::Info,
        crtc: crtc::Handle,
        props: DeviceProps,
    ) -> Result<()> {
        let DeviceProps { handle, dh, primary_gpu, config, devices, space, gpus } = props;

        let Some(device) = devices.get_mut(&node) else {
            bail!("connector on non existing device")
        };

        let dh = dh.clone();
        let mut renderer = gpus.single_renderer(&device.render_node)?;
        let drm_device = device.drm_output_manager.device();


        // NOTE: device informations
        let name = format!("{}-{}",connector.interface().as_str(),connector.interface_id());
        let display_info = display_info::for_connectors(drm_device, connector.handle());
        let make = display_info.as_ref().and_then(|info|info.make()).unwrap_or_else(||"Unknown".into());
        let model = display_info.as_ref().and_then(|info|info.model()).unwrap_or_else(||"Unknown".into());

        tracing::info!(?crtc,"setting up connector {name}");

        // NOTE: non-desktop
        if drm_device
            .get_properties(connector.handle())
            .ok()
            .and_then(|props|{
                let (info,value) = props
                    .into_iter()
                    .filter_map(|(handle,value)|Some((drm_device.get_property(handle).ok()?,value)))
                    .find(|(info,_)|info.name().to_str()==Ok("non-desktop"))?;
                info.value_type().convert_value(value).as_boolean()
            })
            .unwrap_or(false)
        {
            tracing::info!("connector {name} is non-desktop, setting up for leasing");
            if let Some(lease_state) = device.drm_lease_state.as_mut() {
                lease_state.add_connector::<Dilema>(connector.handle(), name, format!("{make} {model}"));
            }
            device.non_desktop_connectors.push((connector.handle(), crtc));
            return Ok(());
        }

        // NOTE: calculate position relative to multiple outputs,
        // presumably multiple workspace, later workspace offset to right side more
        let position = {
            let x = space.outputs().fold(0, |acc, o| acc + space.output_geometry(o).unwrap().size.w);
            (x, 0)
        };

        // NOTE: look for prefered Mode
        let drm_mode = *match connector
            .modes()
            .iter()
            .find(|mode| mode.mode_type().contains(ModeTypeFlags::PREFERRED))
        {
            Some(mode) => mode,
            None => connector.modes().get(0).expect("no modes available on connector"),
        };

        // NOTE: #1 setup Output
        let output = {
            let (w, h) = connector.size().unwrap_or((0,0));
            let subpixel = connector.subpixel().into();
            let physical = PhysicalProperties {
                size: (w as i32,h as i32).into(),
                subpixel, make, model
            };
            Output::new(name, physical)
        };

        // NOTE: #2 apply Output changes, and apply to Space
        {
            output.set_preferred(drm_mode.into());
            output.change_current_state(Some(drm_mode.into()), None, None, Some(position.into()));
            output.user_data().insert_if_missing(||UdevOutputId { crtc, node });
            space.map_output(&output, position);
        }

        // NOTE: #3 create Output Global
        let global = output.create_global::<Dilema>(&dh);

        // NOTE: #4 setup DrmOutput
        let drm_output = {
            let mut planes = drm_device.planes(&crtc)?;

            // nvidia moment
            {
                let driver = drm_device.get_driver()?;
                let is_nvidia = driver.name().eq_ignore_ascii_case("nvidia") ||
                    driver.description().eq_ignore_ascii_case("nvidia");
                if is_nvidia {
                    planes.overlay.clear();
                }
            }

            device
                .drm_output_manager
                .initialize_output::<_, SolidColorRenderElement>(
                    crtc,
                    drm_mode,
                    &[connector.handle()],
                    &output,
                    Some(planes),
                    &mut renderer,
                    &DrmOutputRenderElements::default(),
                )?
        };

        // NOTE: #5 setup DmabufFeedback
        let dmabuf_feedback = drm_output.with_compositor(|compositor|{
            compositor.set_debug_flags(DebugFlags::empty());
            self::get_surface_dmabuf_feedback(*primary_gpu, device.render_node, gpus, compositor.surface())
        });

        // NOTE: #6 setup SurfaceData
        let surface = SurfaceData {
            dh,
            node,
            global,
            render_node: device.render_node,

            drm_output,
            dmabuf_feedback,

            disable_direct_scanout: config.disable_direct_scanout,
        };
        device.surfaces.insert(crtc, surface);

        // kick-off rendering
        handle.insert_idle(move|dilema|{
            render::surface(node, crtc, dilema.clock.now(), dilema);
        });

        Ok(())
    }

    fn connector_disconnected(
        node: DrmNode,
        connector: connector::Info,
        crtc: crtc::Handle,
        props: DeviceProps,
    ) -> Result<()> {
        let DeviceProps { devices, space, gpus, .. } = props;

        let Some(device) = devices.get_mut(&node) else {
            bail!("connector on non existing device")
        };

        if let Some(pos) = device
            .non_desktop_connectors
            .iter()
            .position(|(h, _)| *h == connector.handle())
        {
            // NOTE: if non-desktop remove connector and withdraw drm lease
            device.non_desktop_connectors.remove(pos);
            if let Some(leasing) = &mut device.drm_lease_state {
                leasing.withdraw_connector(connector.handle());
            }
        } else {
            // NOTE: remove Surface and unmap Output from Space
            device.surfaces.remove(&crtc);
            let output = space
                .outputs()
                .find(|output|{
                    output.user_data()
                        .get::<UdevOutputId>()
                        .map(|id|id.node == node && id.crtc == crtc)
                        .unwrap_or(false)
                })
                .cloned();
            if let Some(output) = output {
                space.unmap_output(&output);
            }
        }


        let mut renderer = gpus.single_renderer(&device.render_node).unwrap();
        let _ = device.drm_output_manager.try_to_restore_modifiers::<_, SolidColorRenderElement>(
            &mut renderer,
            &DrmOutputRenderElements::default()
        );

        Ok(())
    }

    fn changed(device_id: u64, props: DeviceProps) -> Result<()> {
        let DeviceProps { handle, dh, primary_gpu, config, devices, space, gpus } = props;

        let node = DrmNode::from_dev_id(device_id)?;

        let Some(device) = devices.get_mut(&node) else {
            bail!("connector on non existing device")
        };

        let scan_result = device.drm_scanner.scan_connectors(device.drm_output_manager.device())?;

        for (conn, crtc) in scan_result.connected {
            let props = DeviceProps { handle, dh, primary_gpu, config, devices, space, gpus };
            if let Some(crtc) = crtc {
                if let Err(err) = self::connector_connected(node, conn, crtc, props) {
                    tracing::error!("{err}");
                }
            }
        }

        for (conn, crtc) in scan_result.disconnected {
            let props = DeviceProps { handle, dh, primary_gpu, config, devices, space, gpus };
            if let Some(crtc) = crtc {
                if let Err(err) = self::connector_disconnected(node, conn, crtc, props) {
                    tracing::error!("{err}");
                }
            }
        }

        Ok(())
    }

    fn removed(device_id: u64, props: DeviceProps) -> Result<()> {
        let DeviceProps { handle, dh, primary_gpu, config, devices, space, gpus } = props;

        let node = DrmNode::from_dev_id(device_id)?;

        let Some(device) = devices.get_mut(&node) else {
            bail!("connector on non existing device")
        };

        let crtcs = device.drm_scanner.crtcs().map(|(info,crtc)|(info.clone(),crtc)).collect::<Vec<_>>();

        for (conn, crtc) in crtcs {
            let props = DeviceProps { handle, dh, primary_gpu, config, devices, space, gpus, };
            if let Err(err) = self::connector_disconnected(node, conn, crtc, props) {
                tracing::error!("{err}");
            }
        }

        if let Some(device) = devices.remove(&node) {
            let DeviceData { drm_lease_state, render_node, token, .. } = device;
            if let Some(mut lease_state) = drm_lease_state {
                lease_state.disable_global::<Dilema>();
            }

            gpus.as_mut().remove_node(&render_node);
            handle.remove(token);
            tracing::debug!("dropping device");
        }

        Ok(())
    }

    // Utils

    pub fn get_surface_dmabuf_feedback(
        primary_gpu: DrmNode,
        render_node: DrmNode,
        gpus: &mut Gpu,
        surface: &DrmSurface,
    ) -> Option<SurfaceDmabufFeedback> {
        let primary_formats = gpus.single_renderer(&primary_gpu).ok()?.dmabuf_formats();
        let render_formats = gpus.single_renderer(&render_node).ok()?.dmabuf_formats();

        let all_render_formats = primary_formats
            .iter()
            .chain(render_formats.iter())
            .copied()
            .collect::<FormatSet>();

        let planes = surface.planes().clone();

        // We limit the scan-out tranche to formats we can also render from
        // so that there is always a fallback render path available in case
        // the supplied buffer can not be scanned out directly
        let planes_formats = surface
            .plane_info()
            .formats
            .iter()
            .copied()
            .chain(planes.overlay.into_iter().flat_map(|p| p.formats))
            .collect::<FormatSet>()
            .intersection(&all_render_formats)
            .copied()
            .collect::<FormatSet>();

        let builder = DmabufFeedbackBuilder::new(primary_gpu.dev_id(), primary_formats);
        let render_feedback = builder
            .clone()
            .add_preference_tranche(render_node.dev_id(), None, render_formats.clone())
            .build()
            .unwrap();

        let scanout_feedback = builder
            .add_preference_tranche(
                surface.device_fd().dev_id().unwrap(),
                Some(zwp_linux_dmabuf_feedback_v1::TrancheFlags::Scanout),
                planes_formats,
            )
            .add_preference_tranche(render_node.dev_id(), None, render_formats)
            .build()
            .unwrap();

        Some(SurfaceDmabufFeedback {
            render_feedback,
            scanout_feedback,
        })
    }
}

mod render {
    use std::time::{Duration, Instant};

    use smithay::{backend::{drm::{compositor::FrameFlags, DrmAccessError, DrmError}, renderer::element::RenderElementStates, SwapBuffersError}, reexports::{calloop::timer::{TimeoutAction, Timer}, wayland_protocols::wp::presentation_time::server::wp_presentation_feedback}, utils::{Monotonic, Time}, wayland::presentation::Refresh};

    use crate::UdevOutputId;
    use super::*;

    pub fn node(
        node: DrmNode,
        crtc: Option<crtc::Handle>,
        frame_target: Time<Monotonic>,
        dilema: &mut Dilema
    ) {
        let device_backend = match dilema.devices.get_mut(&node) {
            Some(backend) => backend,
            None => {
                tracing::error!("trying to render on non-existent backend {node}");
                return;
            }
        };

        match crtc {
            Some(crtc) => {
                self::surface(node, crtc, frame_target, dilema);
            },
            None => {
                let crtcs = device_backend.surfaces.keys().copied().collect::<Vec<_>>();
                for crtc in crtcs {
                    self::surface(node, crtc, frame_target, dilema);
                }
            },
        }
    }

    pub fn surface(
        node: DrmNode,
        crtc: crtc::Handle,
        frame_target: Time<Monotonic>,
        dilema: &mut Dilema
    ) {
        // let Some(device) = trayle.backend.devices.get_mut(&node) else {
        //     return; //Err(DeviceError::DeviceUntracked(node));
        // };

        let Some(output) = dilema
            .space
            .outputs()
            .find(|o|{
                o.user_data().get::<UdevOutputId>()==Some(&UdevOutputId { node, crtc })
            })
            .cloned()
        else {
            // somehow called with invalid output
            return;
        };

        // self.pre_repaint(&output, frame_target);

        let Some(device) = dilema.devices.get_mut(&node) else {
            return;
        };

        let Some(surface) = device.surfaces.get_mut(&crtc) else {
            return;
        };

        let start = Instant::now();

        /*
        // TODO: get scale from render surface when supporting HiDPI
        let frame = trayle
            .backend
            .pointer_image
            .get_image(1 /*scale*/, trayle.frontend.clock.now().into());
        */

        let render_node = surface.render_node;
        let primary_gpu = dilema.primary_gpu;
        let mut renderer = if primary_gpu == render_node {
            dilema.gpus.single_renderer(&render_node)
        } else {
            let format = surface.drm_output.format();
            dilema.gpus.renderer(&primary_gpu, &render_node, format)
        }.unwrap();

        /*
        let pointer_images = &mut self.backend.pointer_images;
        let pointer_image = pointer_images
            .iter()
            .find_map(|(image,texture)|{
                if image == &frame {
                    Some(texture.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(||{
                let buffer = MemoryRenderBuffer::from_slice(
                    &frame.pixels_rgba,
                    Fourcc::Argb8888,
                    (frame.width as i32, frame.height as i32),
                    1,
                    Transform::Normal,
                    None,
                );
                pointer_images.push((frame, buffer.clone()));
                buffer
            });
        */

        let result = inner_render_surface(
            surface,
            &mut renderer,
            &dilema.space,
            &output,
            &dilema.config
        );

        let reschedule = match result {
            Ok((has_rendered, states)) => {
                let dmabuf_feedback = surface.dmabuf_feedback.clone();
                // self.post_repaint(&output, frame_target, dmabuf_feedback, &states);
                !has_rendered
            }
            Err(err) => {
                tracing::warn!("failed to render: {err:?}");
                false
                // match err {
                //     SwapBuffersError::AlreadySwapped => false,
                //     SwapBuffersError::TemporaryFailure(err) => match err.downcast_ref::<DrmError>() {
                //         Some(DrmError::DeviceInactive) => true,
                //         Some(DrmError::Access(DrmAccessError { source, .. })) => {
                //             source.kind() == std::io::ErrorKind::PermissionDenied
                //         }
                //         _ => false,
                //     },
                //     SwapBuffersError::ContextLost(err) => match err.downcast_ref::<DrmError>() {
                //         Some(DrmError::TestFailed(_)) => {
                //             // reset the complete state, disabling all connectors and planes in case we hit a test failed
                //             // most likely we hit this after a tty switch when a foreign master changed CRTC <-> connector bindings
                //             // and we run in a mismatch
                //             device
                //                 .drm_output_manager
                //                 .device_mut()
                //                 .reset_state()
                //                 .expect("failed to reset drm device");
                //             true
                //         }
                //         _ => panic!("rendering loop lost: {err}"),
                //     },
                // }
            }
        };

        if reschedule {
            let output_refresh = match output.current_mode() {
                Some(mode) => mode.refresh,
                None => return,
            };

            // if rescheduling, rendering either hit a temporary failure or did not cause any
            // damage on the output.
            // in this case, just re-scehdule a repaint after approx.
            // one frame to re-test for damage
            let next_frame_target = frame_target + Duration::from_millis(1_000_000 / output_refresh as u64);
            let reschedule_timeout = Duration::from(
                next_frame_target).saturating_sub(dilema.clock.now().into()
            );

            tracing::trace!("reschedule repaint timer with delay {reschedule_timeout:?} on {crtc:?}");

            let timer = Timer::from_duration(reschedule_timeout);
            dilema.handle.insert_source(timer, move|_,_,trayle|{
                tracing::trace!("render::timer");
                self::node(node, Some(crtc), next_frame_target, trayle);
                TimeoutAction::Drop
            })
            .expect("failed to reschedule frame timer");
        } else {
            let elapsed = start.elapsed();
            tracing::trace!(?elapsed, "rendered surface");
        }
    }

    fn inner_render_surface<'a>(
        surface: &'a mut SurfaceData,
        renderer: &mut UdevRenderer<'a>,
        space: &Space<Window>,
        output: &Output,
        config: &Config,
    ) -> Result<(bool, RenderElementStates)> {
        // let output_geometry = space.output_geometry(output).unwrap();
        // let scale = Scale::from(output.current_scale().fractional_scale());
        //
        // let mut custom_elements = Vec::<String>::new();

        /*

        if output_geometry.to_f64().contains(pointer_location) {
            let cursor_hotspot = if let CursorImageStatus::Surface(ref surface) = cursor_status {
                compositor::with_states(surface, |states|{
                    states.data_map
                        .get::<Mutex<CursorImageAttributes>>()
                        .unwrap()
                        .lock()
                        .unwrap()
                        .hotspot
                })
            } else {
                (0,0).into()
            };

            let cursor_pos = pointer_location - output_geometry.loc.to_f64();

            // set cursor
            pointer_element.set_buffer(pointer_image.clone());

            // draw the cursor as relevant
            {
                // reset the cursor if the surface is no longer alive
                let mut reset = false;
                if let CursorImageStatus::Surface(ref surface) = *cursor_status {
                    reset = !surface.alive();
                }
                if reset {
                    *cursor_status = CursorImageStatus::default_named();
                }

                pointer_element.set_status(cursor_status.clone());
            }

            custom_elements.extend(
                pointer_element.render_elements(
                    renderer,
                    (cursor_pos - cursor_hotspot.to_f64())
                        .to_physical(scale)
                        .to_i32_round(),
                    scale,
                    1.0
                ),
            );

            // draw the dnd icon if applicable
            {
                if let Some(icon) = dnd_icon.as_ref() {
                    let dnd_icon_pos = (cursor_pos + icon.offset.to_f64())
                        .to_physical(scale)
                        .to_i32_round();
                    if icon.surface.alive() {
                        custom_elements.extend(AsRenderElements::<UdevRenderer<'a>>::render_elements(
                            &SurfaceTree::from_surface(&icon.surface),
                            renderer,
                            dnd_icon_pos,
                            scale,
                            1.0
                        ));
                    }
                }
            }
        }
        */

        let elements = self::elements::outputs(output, space, renderer);

        let frame_mode = match surface.disable_direct_scanout {
            true => FrameFlags::empty(),
            false => FrameFlags::DEFAULT,
        };

        let (rendered, render_elements_states) = surface
            .drm_output
            .render_frame(renderer, &elements, config.clear_color, frame_mode)
            .map(|render_frame_result|{
                // renderer_sync feature
                (!render_frame_result.is_empty,render_frame_result.states)
            })
            .unwrap();

        for window in space.elements() {
            window.with_surfaces(|surface,states|{
                smithay::desktop::utils::update_surface_primary_scanout_output(
                    surface, output, states, &render_elements_states,
                    smithay::backend::renderer::element::default_primary_scanout_output_compare
                );
            });
        }

        let map = smithay::desktop::layer_map_for_output(output);
        for layer_surface in map.layers() {
            layer_surface.with_surfaces(|surface,states|{
                smithay::desktop::utils::update_surface_primary_scanout_output(
                    surface, output, states, &render_elements_states,
                    smithay::backend::renderer::element::default_primary_scanout_output_compare
                );
            });
        }



        if rendered {
            let mut output_presentation_feedback = OutputPresentationFeedback::new(output);
            for window in space.elements() {
                if space.outputs_for_element(window).contains(output) {
                    window.take_presentation_feedback(
                        &mut output_presentation_feedback,
                        smithay::desktop::utils::surface_primary_scanout_output,
                        |surface,_| smithay::desktop::utils::surface_presentation_feedback_flags_from_states(
                            surface, &render_elements_states
                        ),
                    );
                }
            }
            let map = smithay::desktop::layer_map_for_output(output);
            for layer_surface in map.layers() {
                layer_surface.take_presentation_feedback(
                    &mut output_presentation_feedback,
                    smithay::desktop::utils::surface_primary_scanout_output,
                    |surface,_| smithay::desktop::utils::surface_presentation_feedback_flags_from_states(
                        surface, &render_elements_states
                    ),
                );
            }
            surface
                .drm_output
                .queue_frame(Some(output_presentation_feedback))
                .unwrap();
        }

        Ok((rendered,render_elements_states))
    }

    pub fn frame_finish(
        node: DrmNode,
        crtc: crtc::Handle,
        meta: &mut Option<DrmEventMetadata>,
        dilema: &mut Dilema,
    ) {
        let device_backend = match dilema.devices.get_mut(&node) {
            Some(some) => some,
            None => {
                tracing::error!("attempt to finish frame on non-existent crtc {crtc:?}");
                return;
            },
        };

        let surface = match device_backend.surfaces.get_mut(&crtc) {
            Some(some) => some,
            None => {
                tracing::error!("attempt to finish frame on non-existent crtc {crtc:?}");
                return;
            },
        };

        let Some(output) = dilema.space.outputs().find(|o|{
            o.user_data().get::<UdevOutputId>() == Some(&UdevOutputId {
                node: surface.node, crtc
            })
        }).cloned()
        else {
            return;
        };

        let tp = meta.as_ref().and_then(|meta|match meta.time {
            smithay::backend::drm::DrmEventTime::Monotonic(tp) => Some(tp),
            smithay::backend::drm::DrmEventTime::Realtime(_) => None,
        });

        let seq = meta.as_ref().map(|meta|meta.sequence).unwrap_or(0);

        let (clock,flags) = match tp {
            Some(tp) => (tp.into(),
                wp_presentation_feedback::Kind::Vsync
                | wp_presentation_feedback::Kind::HwClock
                | wp_presentation_feedback::Kind::HwCompletion
            ),
            None => (dilema.clock.now(),
                wp_presentation_feedback::Kind::Vsync
            ),
        };

        let submit_result = surface
            .drm_output
            .frame_submitted()
            .map_err(Into::<SwapBuffersError>::into);

        let Some(frame_duration) = output
            .current_mode()
            .map(|mode|Duration::from_secs_f64(1_000f64/mode.refresh as f64))
        else {
            return
        };

        let schedule_render = match submit_result {
            Ok(user_data) => {
                if let Some(mut feedback) = user_data.flatten() {
                    feedback.presented(clock, Refresh::Fixed(frame_duration), seq as u64, flags);
                }
                true
            },
            Err(err) => {
                tracing::warn!("rendering error: {err:?}");
                match err {
                    SwapBuffersError::AlreadySwapped => true,
                    // If the device has been deactivated do not reschedule, this will be done
                    // by session resume
                    SwapBuffersError::TemporaryFailure(err)
                        if matches!(err.downcast_ref::<DrmError>(),Some(&DrmError::DeviceInactive)) =>
                    {
                        false
                    }
                    SwapBuffersError::TemporaryFailure(err) => matches!{
                        err.downcast_ref::<DrmError>(),
                        Some(DrmError::Access(DrmAccessError { source, .. }))
                            if source.kind() == std::io::ErrorKind::PermissionDenied,
                    },
                    SwapBuffersError::ContextLost(err) => panic!("rendering loop lost: {err}")
                }
            },
        };

        if schedule_render {
            let next_frame_target = clock + frame_duration;

            // What are we trying to solve by introducing a delay here:
            //
            // Basically it is all about latency of client provided buffers.
            // A client driven by frame callbacks will wait for a frame callback
            // to repaint and submit a new buffer. As we send frame callbacks
            // as part of the repaint in the compositor the latency would always
            // be approx. 2 frames. By introducing a delay before we repaint in
            // the compositor we can reduce the latency to approx. 1 frame + the
            // remaining duration from the repaint to the next VBlank.
            //
            // With the delay it is also possible to further reduce latency if
            // the client is driven by presentation feedback. As the presentation
            // feedback is directly sent after a VBlank the client can submit a
            // new buffer during the repaint delay that can hit the very next
            // VBlank, thus reducing the potential latency to below one frame.
            //
            // Choosing a good delay is a topic on its own so we just implement
            // a simple strategy here. We just split the duration between two
            // VBlanks into two steps, one for the client repaint and one for the
            // compositor repaint. Theoretically the repaint in the compositor should
            // be faster so we give the client a bit more time to repaint. On a typical
            // modern system the repaint in the compositor should not take more than 2ms
            // so this should be safe for refresh rates up to at least 120 Hz. For 120 Hz
            // this results in approx. 3.33ms time for repainting in the compositor.
            // A too big delay could result in missing the next VBlank in the compositor.
            //
            // A more complete solution could work on a sliding window analyzing past repaints
            // and do some prediction for the next repaint.
            let repaint_delay = Duration::from_secs_f64(frame_duration.as_secs_f64() * 0.6f64);

            let timer = if dilema.primary_gpu != surface.render_node {
                // However, if we need to do a copy, that might not be enough.
                // (And without actual comparision to previous frames we cannot really know.)
                // So lets ignore that in those cases to avoid thrashing performance.
                tracing::trace!("scheduling repaint timer immediately on {crtc:?}");
                Timer::immediate()
            } else {
                tracing::trace!("scheduling repaint timer with delay {repaint_delay:?} on {crtc:?}");
                Timer::from_duration(repaint_delay)
            };

            dilema.handle.insert_source(timer, move|_,_,trayle|{
                tracing::trace!("frame_finish::reschedule");
                render::node(node, Some(crtc), next_frame_target, trayle);
                TimeoutAction::Drop
            }).expect("failed to schedule frame timer");
        }
    }

    pub mod elements {
        use smithay::{backend::renderer::{element::{memory::MemoryRenderBufferRenderElement, surface::WaylandSurfaceRenderElement, utils::{CropRenderElement, RelocateRenderElement, RescaleRenderElement}, Wrap}, ImportAll, ImportMem, Renderer}, desktop::space::SpaceRenderElements};

        use super::*;

        smithay::render_elements! {
            pub WindowRenderElement<R> where R: ImportAll + ImportMem;
            Window=WaylandSurfaceRenderElement<R>,
            Decoration=SolidColorRenderElement,
        }

        smithay::render_elements! {
            pub PointerRenderElement<R> where R: ImportAll + ImportMem;
            Surface=WaylandSurfaceRenderElement<R>,
            Memory=MemoryRenderBufferRenderElement<R>,
        }

        smithay::render_elements! {
            pub CustomRenderElements<R> where R: ImportAll + ImportMem;
            Pointer=PointerRenderElement<R>,
            Surface=WaylandSurfaceRenderElement<R>,
        }

        smithay::render_elements! {
            pub OutputRenderElements<R, E> where R: ImportAll + ImportMem;
            Space=SpaceRenderElements<R, E>,
            Window=Wrap<E>,
            Custom=CustomRenderElements<R>,
            Preview=CropRenderElement<RelocateRenderElement<RescaleRenderElement<WindowRenderElement<R>>>>,
        }

        /// output elements
        pub fn outputs<R>(
            output: &Output,
            space: &Space<Window>,
            renderer: &mut R,
        ) -> Vec<SpaceRenderElements<R, WaylandSurfaceRenderElement<R>>>
        where
            R: Renderer + ImportAll + ImportMem,
            R::TextureId: Clone + 'static,
        {
            let space_elements = smithay::desktop::space::space_render_elements::<_, Window, _>(
                renderer,
                [space],
                output,
                1.0,
            )
            .unwrap();

            space_elements
        }
    }

}

mod drm_scanner {
    use super::*;

    pub struct DrmScanner {
        connectors: HashMap<connector::Handle, connector::Info>,
        crtcs: HashMap<connector::Handle, crtc::Handle>,
    }

    impl DrmScanner {
        pub fn new() -> DrmScanner {
            Self {
                connectors: Default::default(),
                crtcs: Default::default(),
            }
        }

        pub fn connectors(&self) -> &HashMap<connector::Handle, connector::Info> {
            &self.connectors
        }

        pub fn crtcs(&self) -> impl Iterator<Item = (&connector::Info, crtc::Handle)> {
            self.connectors()
                .iter()
                .filter_map(|(handle,info)|Some((info,self.crtcs.get(handle).copied()?)))
        }

        fn is_taken(&self, crtc: &crtc::Handle) -> bool {
            self.crtcs.values().any(|c|c == crtc)
        }

        fn is_available(&self, crtc: &crtc::Handle) -> bool {
            !self.is_taken(crtc)
        }

        fn restored_for_connector(
            &self,
            drm: &impl ControlDevice,
            connector: &connector::Info,
        ) -> Option<crtc::Handle> {
            let crtc = drm.get_encoder(connector.current_encoder()?).ok()?.crtc()?;
            self.is_available(&crtc).then_some(crtc)
        }

        fn pick_next_available_for_connector(
            &self,
            drm: &impl ControlDevice,
            connector: &connector::Info,
        ) -> Option<crtc::Handle> {
            let res_handle = drm.resource_handles().ok()?;
            connector
                .encoders()
                .iter()
                .flat_map(|enc|drm.get_encoder(*enc))
                .find_map(|enc|{
                    res_handle
                        .filter_crtcs(enc.possible_crtcs())
                        .into_iter()
                        .find(|crtc|self.is_available(crtc))
                })
        }

        pub fn scan_connectors(&mut self, drm: &impl ControlDevice) -> std::io::Result<DrmScanResult> {
            let resource_handle = drm.resource_handles()?;
            let connector_handles = resource_handle
                .connectors
                .iter()
                .filter_map(|conn| drm.get_connector(*conn, true).ok());

            let mut connected = vec![];
            let mut disconnected = vec![];

            for conn in connector_handles {
                let conn_handle = conn.handle();
                let after_state = conn.state();
                let old = self.connectors.insert(conn.handle(),conn.clone());
                let state_change = (old.map(|e|e.state()), after_state);

                use connector::State::*;

                if matches!(state_change, (Some(Disconnected|Unknown)|None, Connected)) {
                    // connected
                    if !self.crtcs.contains_key(&conn_handle) {
                        if let Some(crtc) = self.restored_for_connector(drm, &conn) {
                            self.crtcs.insert(conn_handle, crtc);
                        } else {
                            if let Some(crtc) = self.pick_next_available_for_connector(drm, &conn) {
                                self.crtcs.insert(conn_handle, crtc);
                            }
                        }
                    }

                    let crtc = self.crtcs.get(&conn_handle).copied();
                    connected.push((conn, crtc));
                }

                else if matches!(state_change,(Some(Connected), Disconnected)) {
                    // disconnected
                    let conn_handle = conn.handle();
                    let crtc = self.crtcs.get(&conn_handle).copied();
                    self.crtcs.remove(&conn_handle);
                    disconnected.push((conn,crtc));
                }

                else {
                    tracing::warn!(
                        "unhandled connector state changes from {:?} to {:?}",
                        state_change.0,
                        state_change.1
                    );
                }
            }

            Ok(DrmScanResult {
                connected,
                disconnected,
            })
        }
    }

    /// Result of [`DrmScanner::scan_connectors`]
    ///
    /// You can use `added` and `removed` fields of this result manually,
    /// or you can just iterate (using [`IntoIterator`] or [`DrmScanResult::iter`])
    /// over this result to get [`DrmScanEvent`].
    #[derive(Debug, Default, Clone)]
    pub struct DrmScanResult {
        /// Connectors that got plugged in since last scan
        pub connected: Vec<(connector::Info, Option<crtc::Handle>)>,
        /// Connectors that got unplugged since last scan
        pub disconnected: Vec<(connector::Info, Option<crtc::Handle>)>,
    }
}

mod display_info {
    use super::*;
    use libdisplay_info::info::Info as DisplayInfo;

    pub fn for_connectors(device: &impl ControlDevice, connector: connector::Handle) -> Option<DisplayInfo> {
        let props = device.get_properties(connector).ok()?;

        let (info,value) = props.into_iter()
            .filter_map(|(handle,value)|{
                let info = device.get_property(handle).ok()?;
                Some((info,value))
            })
        .find(|(info,_)|info.name().to_str()==Ok("EDID"))?;

        let blob = info.value_type().convert_value(value).as_blob()?;
        let data = device.get_property_blob(blob).ok()?;

        DisplayInfo::parse_edid(&data).ok()
    }

}



smithay::delegate_compositor!(Dilema);

impl CompositorHandler for Dilema {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        callbacks::compositor_commit(surface, self);
    }
}

smithay::delegate_xdg_shell!(Dilema);

impl XdgShellHandler for Dilema {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let window = Window::new_wayland_window(surface);
        self.space.map_element(window, (0,0), false);
    }

    fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {
        tracing::warn!("XdgShellHandler::new_popup is not yet implemented");
    }

    fn grab(&mut self, _surface: PopupSurface, _seat: WlSeat, _serial: Serial) {
        tracing::warn!("XdgShellHandler::grab is not yet implemented");
    }

    fn reposition_request(&mut self, _surface: PopupSurface, _positioner: PositionerState, _token: u32) {
        tracing::warn!("XdgShellHandler::reposition_request is not yet implemented");
    }
}

smithay::delegate_seat!(Dilema);

impl SeatHandler for Dilema {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }
}

smithay::delegate_shm!(Dilema);

impl ShmHandler for Dilema {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl BufferHandler for Dilema {
    fn buffer_destroyed(&mut self, _buffer: &WlBuffer) { }
}

smithay::delegate_dmabuf!(Dilema);

impl DmabufHandler for Dilema {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        &mut self.dmabuf_state
    }

    fn dmabuf_imported(&mut self, _global: &DmabufGlobal, _dmabuf: Dmabuf, _notifier: ImportNotifier) {
        tracing::warn!("DmabufHandler::dmabuf_imported is not yet implemented");
    }
}

smithay::delegate_output!(Dilema);

impl OutputHandler for Dilema {
    fn output_bound(&mut self, _output: Output, _wl_output: WlOutput) {}
}

smithay::delegate_drm_lease!(Dilema);

impl DrmLeaseHandler for Dilema {
    fn drm_lease_state(&mut self, node: DrmNode) -> &mut DrmLeaseState {
        self.devices.get_mut(&node).unwrap().drm_lease_state.as_mut().unwrap()
    }

    fn lease_request(
        &mut self,
        node: DrmNode,
        request: DrmLeaseRequest,
    ) -> Result<DrmLeaseBuilder, LeaseRejected> {
        let device = self.devices.get(&node).ok_or_else(LeaseRejected::default)?;

        let drm_device = device.drm_output_manager.device();
        let mut builder = DrmLeaseBuilder::new(drm_device);
        for conn in request.connectors {
            let Some(&(_, crtc)) = device
                .non_desktop_connectors
                .iter()
                .find(|(c, _)| *c == conn)
            else {
                tracing::warn!("lease request denied for desktop connector");
                return Err(LeaseRejected::default());
            };

            builder.add_connector(conn);
            builder.add_crtc(crtc);
            let planes = drm_device.planes(&crtc).map_err(LeaseRejected::with_cause)?;
            let (primary_plane, primary_plane_claim) = planes
                .primary
                .iter()
                .find_map(|plane|{
                    drm_device
                        .claim_plane(plane.handle, crtc)
                        .map(|claim|(plane,claim))
                })
                .ok_or_else(LeaseRejected::default)?;
            builder.add_plane(primary_plane.handle, primary_plane_claim);
            if let Some((cursor,claim)) = planes.cursor.iter().find_map(|plane|{
                drm_device.claim_plane(plane.handle, crtc).map(|claim|(plane,claim))
            }) {
                builder.add_plane(cursor.handle, claim);
            }
        }

        Ok(builder)
    }

    fn new_active_lease(&mut self, node: DrmNode, lease: DrmLease) {
        let device = self.devices.get_mut(&node).unwrap();
        device.active_leases.push(lease);
    }

    fn lease_destroyed(&mut self, node: DrmNode, lease_id: u32) {
        let device = self.devices.get_mut(&node).unwrap();
        device.active_leases.retain(|l| l.id() != lease_id);
    }
}

smithay::delegate_drm_syncobj!(Dilema);

impl DrmSyncobjHandler for Dilema {
    fn drm_syncobj_state(&mut self) -> &mut DrmSyncobjState {
        self.drm_syncobj.as_mut().expect("gpu does not support DRM syncobj protocol")
    }
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}
