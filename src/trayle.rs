//! entrypoints of the codebase
//!
//! [`Trayle`] is the main state of the compositor
//!
//! other modules contains domain specific logic
//!
//! - [`callbacks`], event loop callbacks
//! - [`device`], `udev` device event handlers
//! - [`render`], rendering logic
//! - [`utils`], utilities for combining multiple domain logic
//!
use crate::{
    backend::{Backend, BackendSources},
    config::{Config, SUPPORTED_FORMATS, SUPPORTED_FORMATS_8BIT_ONLY},
    frontend::{Frontend, FrontendSources, SurfaceDmabufFeedback},
    utils::{
        display_info,
        drm_scanner::{DrmScanEvent, DrmScanner},
    },
};
use anyhow::{Context, Result};
use smithay::{
    backend::{
        allocator::{
            format::FormatSet,
            gbm::{GbmAllocator, GbmBufferFlags, GbmDevice},
        },
        drm::{
            compositor::FrameFlags,
            output::{DrmOutput, DrmOutputManager, DrmOutputRenderElements},
            CreateDrmNodeError, DrmAccessError, DrmDevice, DrmDeviceFd,
            DrmError, DrmEvent, DrmEventMetadata, DrmNode, NodeType,
        },
        egl::{self, EGLDevice, EGLDisplay},
        libinput::LibinputInputBackend,
        renderer::{
            element::{
                memory::MemoryRenderBufferRenderElement,
                solid::SolidColorRenderElement,
                surface::WaylandSurfaceRenderElement,
                utils::{CropRenderElement, RelocateRenderElement, RescaleRenderElement},
                RenderElementStates, Wrap,
            },
            gles::GlesRenderer,
            multigpu::{gbm::GbmGlesBackend, GpuManager},
            utils as renderer_utils, ImportAll, ImportDma, ImportEgl, ImportMem, ImportMemWl,
        },
        session::{
            libseat,
            Event as SessionEvent, Session,
        },
        udev::{UdevBackend, UdevEvent},
        SwapBuffersError,
    },
    desktop::{space::SpaceRenderElements, utils::OutputPresentationFeedback, Space, Window},
    input::Seat,
    output::{Mode as WlMode, Output, PhysicalProperties},
    reexports::{
        calloop::{
            generic::{Generic, NoIoDrop}, timer::{TimeoutAction, Timer}, EventLoop, Interest, LoopHandle, LoopSignal, Mode as FdMode, PostAction, RegistrationToken
        },
        drm::{
            control::{connector, crtc, Device as _, ModeTypeFlags},
            Device as _,
        },
        rustix::fs::OFlags,
        wayland_protocols::wp::{
            linux_dmabuf::zv1::server::zwp_linux_dmabuf_feedback_v1,
            presentation_time::server::wp_presentation_feedback,
        },
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason, GlobalId},
            protocol::wl_surface::WlSurface,
            Display, DisplayHandle,
        },
    },
    utils::{Clock, Monotonic, Time},
    wayland::{
        compositor::{self, CompositorClientState},
        dmabuf::{DmabufFeedbackBuilder, DmabufState},
        drm_lease::{DrmLease, DrmLeaseState},
        drm_syncobj::DrmSyncobjState,
        presentation::Refresh,
        shell::xdg::XdgToplevelSurfaceData,
    },
};

use std::{collections::HashMap, os::unix::net::UnixStream, path::Path, sync::Arc, time::{Duration, Instant}};

type InputEvent = smithay::backend::input::InputEvent<LibinputInputBackend>;

/// the main structs that holds all states
///
/// to construct `Trayle`, see [`Trayle::setup`]
pub struct Trayle {
    pub config: Config,
    pub frontend: Frontend,
    pub backend: Backend,
    pub seat: Seat<Trayle>,
    pub handle: LoopHandle<'static, Trayle>,
    pub signal: LoopSignal,
    pub dh: DisplayHandle,
    pub clock: Clock<Monotonic>,
}

impl Trayle {
    /// construct new [`Trayle`]
    ///
    /// event loop are constructed separately, so the caller is responsible for actually running it
    ///
    /// - setup [`Display`], its the core of a wayland compositor, see its documentation for detail
    /// - setup [`Config`], it holds user configurations, see [`Config::setup`] for more detail
    /// - setup [`Frontend`], it holds wayland protocol states, see [`Frontend::setup`] for more detail
    /// - setup [`Backend`], it holds os interaction states, see [`Backend::setup`] for more detail
    /// - setup [`Seat`], it require [`Frontend::seat_state`] and [`Backend::seat`] to setup
    ///
    /// now that `Trayle` is fully constructed, further setup is called in
    /// [`Trayle::setup_bindings`], see its documentation for more detail
    ///
    /// [`Frontend`] and [`Backend`] setup also return event sources as [`FrontendSources`] and
    /// [`BackendSources`] respectively, registered to event loop with its corresponding
    /// callbacks from [`callbacks`] module
    ///
    /// [`Seat`]: smithay::input::Seat
    /// [`SeatState::new_wl_seat`]: smithay::input::SeatState::new_wl_seat
    pub fn setup(event_loop: &mut EventLoop<'static, Trayle>) -> Result<Trayle> {
        let display = Display::<Trayle>::new().context("failed to setup display")?;
        let handle = event_loop.handle();
        let signal = event_loop.get_signal();
        let dh = display.handle();
        let clock = Clock::new();

        // states
        let config = Config::setup()?;
        let (mut frontend, frontend_sources) = Frontend::setup(&dh)?;
        let (backend, backend_sources) = Backend::setup(&dh)?;

        let seat = frontend.seat_state.new_wl_seat(&dh, &backend.seat);

        let mut trayle = Trayle {
            config,
            frontend,
            backend,
            seat,
            handle,
            signal,
            dh,
            clock,
        };

        trayle.setup_bindings(&backend_sources.udev)?;

        let handle = event_loop.handle();
        let display = Generic::new(display, Interest::READ, FdMode::Edge);

        let FrontendSources { socket } = frontend_sources;
        let BackendSources { session, input, udev } = backend_sources;

        handle.insert_source(socket, callbacks::socket).unwrap();
        handle.insert_source(session, callbacks::session).unwrap();
        handle.insert_source(input, callbacks::input).unwrap();
        handle.insert_source(udev, device::handle_udev).unwrap();
        handle.insert_source(display, callbacks::display).unwrap();

        tracing::info!("setup complete");

        Ok(trayle)
    }

    /// setups inside this function is required trayle to be fully constructed
    ///
    /// - setup [`UdevBackend`], and retrieve available drm devices, see [`UdevBackend::device_list`]
    /// - setup [`SurfaceDmabufFeedback`] for every drm devices via [`utils::get_surface_dmabuf_feedback`]
    /// - update [`ShmState`] formats via renderer through [`ImportMemWl::shm_formats`]
    /// - try enabling EGL hardware-acceleration for renderer through [`ImportEgl::bind_wl_display`]
    /// - expose syncobj protocol if supported by primary gpu by setting up [`DrmSyncobjState`],
    ///   see [`smithay::wayland::drm_syncobj`]
    ///
    /// [`ShmState`]: smithay::wayland::shm::ShmState
    fn setup_bindings(&mut self, udev: &UdevBackend) -> Result<()> {
        // udev
        for (device_id, path) in udev.device_list() {
            let event = UdevEvent::Added { device_id, path: path.to_path_buf() };
            device::handle_udev(event, &mut (), self);
        }


        // update each drm surface dmabuf feedback
        for device_data in self.backend.devices.values_mut() {
            for surface_data in device_data.surfaces.values_mut() {
                if surface_data.dmabuf_feedback.is_some() {
                    continue;
                }
                surface_data.dmabuf_feedback = surface_data.drm_output.with_compositor(|compositor|{
                    utils::get_surface_dmabuf_feedback(
                        self.backend.primary_gpu,
                        surface_data.render_node,
                        &mut self.backend.gpus,
                        compositor.surface(),
                    )
                });
            }
        }


        let mut renderer = self
            .backend
            .gpus
            .single_renderer(&self.backend.primary_gpu)
            .expect("failed to get primary renderer");

        // setup dmabuf support with format list from primary gpu
        let dmabuf_formats = ImportDma::dmabuf_formats(&renderer);
        let feedback = DmabufFeedbackBuilder::new(self.backend.primary_gpu.dev_id(), dmabuf_formats)
            .build()
            .unwrap();
        let mut dmabuf_state = DmabufState::new();
        let global = dmabuf_state.create_global_with_default_feedback::<Trayle>(&self.dh, &feedback);
        self.backend.dmabuf_state.write((dmabuf_state, global));


        // setup shared memory formats
        self.frontend.shm_state.update_formats(ImportMemWl::shm_formats(&renderer));


        // try to enable EGL hardware-acceleration
        match ImportEgl::bind_wl_display(&mut renderer, &self.dh) {
            Ok(_) => tracing::info!("EGL hardware-acceleration enabled"),
            Err(err) => tracing::info!("EGL hardware-acceleration disabled, {err}"),
        };


        // expose syncobj protocol if supported by primary gpu
        'syncobj: {
            let Some(primary_node) = self.backend.primary_gpu
                .node_with_type(NodeType::Primary)
                .and_then(Result::ok)
            else {
                break 'syncobj;
            };

            let Some(device) = self.backend.devices.get(&primary_node) else {
                break 'syncobj;
            };

            let import_device = device.drm_output_manager.device().device_fd().clone();

            if !smithay::wayland::drm_syncobj::supports_syncobj_eventfd(&import_device) {
                break 'syncobj;
            }

            self.backend.syncobj_state.replace(DrmSyncobjState::new::<Trayle>(&self.dh, import_device));

            let device_path = device.render_node.dev_path().unwrap_or_default();
            tracing::info!("drm device {device_path:?} syncobj_eventfd supported",);
        }

        Ok(())
    }

    /// refresh internal state
    ///
    /// need to be called periodically
    ///
    /// this can be a callback for an event loop run
    pub fn refresh(&mut self) {
        self.frontend.space.refresh();
        // state.popups.refresh();

        if let Err(err) = self.dh.flush_clients() {
            tracing::error!("failed to flush clients in display handle: {err}");
        }
    }
}


/// contain functions that called on smithay's handler traits
impl Trayle {
    /// called on [`CompositorHandler::commit`] in [`crate::handlers::compositor`]
    ///
    /// [`CompositorHandler::commit`]: compositor::CompositorHandler::commit
    pub fn surface_commit(&mut self, surface: &WlSurface) {
        // smithay take over buffer management
        renderer_utils::on_commit_buffer_handler::<Self>(surface);

        // idk
        if let Err(err) = self.backend.early_import(surface) {
            tracing::error!("{err}");
        };

        if !compositor::is_sync_subsurface(surface) {
            let mut root_surface =
                compositor::get_parent(surface).unwrap_or_else(|| surface.clone());
            while let Some(parent) = compositor::get_parent(&root_surface) {
                root_surface = parent;
            }

            let root_window = self
                .frontend
                .space
                .elements()
                .find(|window| window.toplevel().unwrap().wl_surface() == &root_surface);

            if let Some(root_window) = root_window {
                // call to action
                root_window.on_commit();
            }
        }


        let current = self.frontend.space.elements().find_map(|window| {
            let toplevel = window.toplevel()?;
            (toplevel.wl_surface() == surface).then_some((toplevel,window))
        });

        if let Some((toplevel_surface, _window)) = current {
            // xdg
            if compositor::with_states(surface, |state| {
                state
                    .data_map
                    .get::<XdgToplevelSurfaceData>()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .initial_configure_sent
            }) {
                // call to action
                toplevel_surface.send_configure();
            }

            // eg: popup commit

            // eg: dnd
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

/// each connected drm device
pub struct DeviceData {
    pub drm_loop_token: RegistrationToken,
    pub drm_output_manager: DrmOutputManager<
        GbmAllocator<DrmDeviceFd>,
        GbmDevice<DrmDeviceFd>,
        Option<OutputPresentationFeedback>,
        DrmDeviceFd,
    >,
    pub drm_scanner: DrmScanner,
    pub non_desktop_connectors: Vec<(connector::Handle, crtc::Handle)>,
    pub render_node: DrmNode,
    pub surfaces: HashMap<crtc::Handle, SurfaceData>,
    pub drm_lease_state: Option<DrmLeaseState>,
    pub active_leases: Vec<DrmLease>,
}

/// surface for each [`DeviceData`]
pub struct SurfaceData {
    pub dh: DisplayHandle,
    pub device_id: DrmNode,
    pub render_node: DrmNode,
    pub global: Option<GlobalId>,
    pub drm_output: DrmOutput<
        GbmAllocator<DrmDeviceFd>,
        GbmDevice<DrmDeviceFd>,
        Option<OutputPresentationFeedback>,
        DrmDeviceFd,
    >,
    pub disable_direct_scanout: bool,
    pub dmabuf_feedback: Option<SurfaceDmabufFeedback>,
}

pub mod device {
    //! `udev` specific handlers
    use super::render::elements::{OutputRenderElements, WindowRenderElement};
    use crate::backend::UdevRenderer;

    use super::*;

    /// handler for [`UdevBackend`] event source
    pub fn handle_udev(event: UdevEvent, _: &mut (), trayle: &mut Trayle) {
        let result = match event {
            UdevEvent::Added { device_id, path } => {
                tracing::info!("device added {path:?}({device_id})");
                device::added(device_id, &path, trayle)
            }
            UdevEvent::Changed { device_id } => {
                tracing::info!("device changed {device_id}");
                device::changed(device_id, trayle)
            }
            UdevEvent::Removed { device_id } => {
                tracing::info!("device removed {device_id}");
                device::removed(device_id, trayle)
            }
        };

        if let Err(err) = result {
            tracing::error!("device error, {err}");
        }
    }

    fn added(device_id: u64, path: &Path, trayle: &mut Trayle) -> Result<(), DeviceError> {
        let node = DrmNode::from_dev_id(device_id)?;

        let flags = OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOCTTY | OFlags::NONBLOCK;
        let fd = Session::open(&mut trayle.backend.session, path, flags)?;
        let fd = DrmDeviceFd::new(fd.into());

        let (drm,drm_source) = DrmDevice::new(fd.clone(), true)?;
        let gbm = GbmDevice::new(fd).map_err(DeviceError::GbmDevice)?;

        let drm_loop_token = trayle.handle.insert_source(
            drm_source, move|e,m,t|callbacks::drm(e,node,m,t)
        ).unwrap();

        let display = unsafe { EGLDisplay::new(gbm.clone()) }?;
        let egldevice = EGLDevice::device_for_display(&display)?;
        let render_node = egldevice.try_get_render_node()?.ok_or(DeviceError::EGLRenderNode)?;

        trayle.backend.gpus.as_mut().add_node(render_node, gbm.clone())?;

        let color_formats = match trayle.config.disable_direct_10bit {
            true => SUPPORTED_FORMATS_8BIT_ONLY,
            false => SUPPORTED_FORMATS,
        };
        let gbm_buffer_flags = GbmBufferFlags::RENDERING | GbmBufferFlags::SCANOUT;
        let allocator = GbmAllocator::new(gbm.clone(), gbm_buffer_flags);
        let mut renderer = trayle.backend.gpus.single_renderer(&render_node).expect("failed to get renderer");
        let render_formats = renderer.as_mut().egl_context().dmabuf_render_formats().clone();

        let drm_output_manager = DrmOutputManager::new(
            drm,
            allocator,
            gbm.clone(),
            Some(gbm),
            color_formats.iter().copied(),
            render_formats
        );

        let drm_lease_state = match DrmLeaseState::new::<Trayle>(&trayle.dh, &node) {
            Ok(ok) => Some(ok),
            Err(err) => {
                tracing::warn!("failed to setup drm lease global for {node}: {err:?}");
                None
            },
        };

        let device_data = DeviceData {
            drm_loop_token,
            render_node,
            drm_output_manager,
            drm_scanner: DrmScanner::new(),
            surfaces: HashMap::new(),
            // known crtcs
            drm_lease_state,
            active_leases: vec![],
            non_desktop_connectors: vec![],
        };

        assert!(trayle.backend.devices.insert(node, device_data).is_none());
        device::changed(device_id, trayle)
    }

    fn changed(device_id: u64, trayle: &mut Trayle) -> Result<(), DeviceError> {
        let node = DrmNode::from_dev_id(device_id)?;
        let Some(device) = trayle.backend.devices.get_mut(&node) else {
            return Err(DeviceError::DeviceUntracked(node));
        };

        let scan_result = device
            .drm_scanner
            .scan_connectors(device.drm_output_manager.device())
            .map_err(DeviceError::ScanConnector)?;

        for event in scan_result {
            match event {
                DrmScanEvent::Connected { connector, crtc: Some(crtc) } => {
                    device::connector_connected(node, connector, crtc, trayle)?;
                },
                DrmScanEvent::Disconnected { connector, crtc: Some(crtc) } => {
                    device::connector_disconnected(node, connector, crtc, trayle)?
                },
                _ => {}
            }
        }

        // fixup window coordinates
        // crate::shell::utils::fixup_positions(&mut self.space, self.pointer.current_location());

        Ok(())
    }

    fn removed(device_id: u64, trayle: &mut Trayle) -> Result<(), DeviceError> {
        let node = DrmNode::from_dev_id(device_id)?;
        let Some(device) = trayle.backend.devices.get_mut(&node) else {
            return Err(DeviceError::DeviceUntracked(node));
        };

        let crtcs = device
            .drm_scanner
            .crtcs()
            .map(|(info, crtc)| (info.clone(), crtc))
            .collect::<Vec<_>>();

        for (connector,crtc) in crtcs {
            if let Err(err) = device::connector_disconnected(node, connector, crtc, trayle) {
                tracing::error!("{err}");
            }
        }

        if let Some(mut device) = trayle.backend.devices.remove(&node) {
            if let Some(mut leasing_global) = device.drm_lease_state.take() {
                leasing_global.disable_global::<Trayle>();
            }

            trayle.backend
                .gpus
                .as_mut()
                .remove_node(&device.render_node);

            trayle.handle.remove(device.drm_loop_token);
        }

        // fixup position
        // crate::shell::utils::fixup_positions(&mut self.space, self.pointer.current_location());

        Ok(())
    }

    fn connector_connected(
        node: DrmNode,
        connector: connector::Info,
        crtc: crtc::Handle,
        trayle: &mut Trayle,
    ) -> Result<(), DeviceError> {
        let Some(device) = trayle.backend.devices.get_mut(&node) else {
            return Err(DeviceError::DeviceUntracked(node));
        };

        let mut renderer = trayle
            .backend
            .gpus
            .single_renderer(&device.render_node)
            .expect("failed to get renderer");

        let output_name = format!("{}-{}",connector.interface().as_str(),connector.interface_id());
        tracing::info!(?crtc,"setting up connector {output_name}");

        let drm_device = device.drm_output_manager.device();

        let non_desktop = drm_device
            .get_properties(connector.handle())
            .ok()
            .and_then(|props|{
                let (info,value) = props
                    .into_iter()
                    .filter_map(|(handle,value)|Some((drm_device.get_property(handle).ok()?,value)))
                    .find(|(info,_)|info.name().to_str()==Ok("non-desktop"))?;
                info.value_type().convert_value(value).as_boolean()
            })
            .unwrap_or(false);

        let display_info = display_info::for_connectors(drm_device, connector.handle());

        let make = display_info.as_ref().and_then(|info|info.make()).unwrap_or_else(||"Unknown".into());
        let model = display_info.as_ref().and_then(|info|info.model()).unwrap_or_else(||"Unknown".into());

        if non_desktop {
            tracing::info!(
                "connector {output_name} is non-desktop, setting up for leasing",
            );

            device.non_desktop_connectors.push((connector.handle(), crtc));
            if let Some(lease_state) = device.drm_lease_state.as_mut() {
                lease_state.add_connector::<Trayle>(
                    connector.handle(), output_name, format!("{make} {model}")
                );
            }

            return Ok(());
        }

        let mode_id = connector
            .modes()
            .iter()
            .position(|mode| mode.mode_type().contains(ModeTypeFlags::PREFERRED))
            .unwrap_or(0);

        let drm_mode = connector.modes()[mode_id];
        let wl_mode = WlMode::from(drm_mode);

        let (phys_w, phys_h) = connector.size().unwrap_or((0,0));
        let physical = PhysicalProperties {
            size: (phys_w as i32,phys_h as i32).into(),
            subpixel: connector.subpixel().into(), make, model
        };
        let output = Output::new(output_name, physical);

        let global = output.create_global::<Trayle>(&trayle.dh);

        let x = trayle.frontend.space.outputs().fold(0, |acc, o| {
            acc + trayle.frontend.space.output_geometry(o).unwrap().size.w
        });

        let position = (x, 0).into();

        output.set_preferred(wl_mode);
        output.change_current_state(Some(wl_mode), None, None, Some(position));
        trayle.frontend.space.map_output(&output, position);

        output.user_data().insert_if_missing(||UdevOutputId { crtc, device_id: node });

        let driver = drm_device.get_driver().map_err(DeviceError::DrmDriver)?;
        let mut planes = drm_device.planes(&crtc)?;

        // nvidia moment
        if driver.name().to_string_lossy().to_lowercase().contains("nvidia") ||
            driver.description().to_string_lossy().to_lowercase().contains("nvidia")
        {
            planes.overlay.clear();
        }

        let drm_output = device.drm_output_manager
            .initialize_output::<_, OutputRenderElements<UdevRenderer<'_>, WindowRenderElement<UdevRenderer<'_>>>>(
                crtc,
                drm_mode,
                &[connector.handle()],
                &output,
                Some(planes),
                &mut renderer,
                &DrmOutputRenderElements::default()
            )
            .map_err(DeviceError::drm_output)?;

        let disable_direct_scanout = std::env::var("TRAYLE_DISABLE_DIRECT_SCANOUT").is_ok();

        let dmabuf_feedback = drm_output.with_compositor(|compositor|{
            compositor.set_debug_flags(trayle.backend.debug_flags);

            utils::get_surface_dmabuf_feedback(
                trayle.backend.primary_gpu,
                device.render_node,
                &mut trayle.backend.gpus,
                compositor.surface(),
            )
        });

        let surface = SurfaceData {
            dh: trayle.dh.clone(),
            device_id: node,
            render_node: device.render_node,
            global: Some(global),
            drm_output,
            disable_direct_scanout,
            dmabuf_feedback,
        };

        device.surfaces.insert(crtc, surface);

        // kick-off rendering
        trayle.handle.insert_idle(move|trayle|{
            render::surface(node, crtc, trayle.clock.now(), trayle);
        });

        Ok(())
    }

    fn connector_disconnected(
        node: DrmNode,
        connector: connector::Info,
        crtc: crtc::Handle,
        trayle: &mut Trayle,
    ) -> Result<(), DeviceError> {
        let Some(device) = trayle.backend.devices.get_mut(&node) else {
            return Err(DeviceError::DeviceUntracked(node));
        };

        if let Some(pos) = device
            .non_desktop_connectors
            .iter()
            .position(|(handle,_)|*handle==connector.handle())
        {
            let _ = device.non_desktop_connectors.remove(pos);
            if let Some(leasing_state) = device.drm_lease_state.as_mut() {
                leasing_state.withdraw_connector(connector.handle());
            }
        } else {
            device.surfaces.remove(&crtc);

            let output = trayle.frontend.space
                .outputs()
                .find(|o|{
                    o.user_data()
                        .get::<UdevOutputId>()
                        .map(|id|id.device_id == node && id.crtc == crtc)
                        .unwrap_or(false)
                })
                .cloned();

            if let Some(output) = output {
                trayle.frontend.space.unmap_output(&output);
            }
        }

        let mut renderer = trayle.backend.gpus.single_renderer(&device.render_node).unwrap();

        let _ = device.drm_output_manager.try_to_restore_modifiers::<_, OutputRenderElements<
            UdevRenderer<'_>,
            WindowRenderElement<UdevRenderer<'_>>,
        >>(
            &mut renderer,
            // FIXME: For a flicker free operation we should return the actual elements for this output..
            // Instead we just use black to "simulate" a modeset :)
            &DrmOutputRenderElements::default(),
        );

        Ok(())
    }

    #[derive(Debug, PartialEq)]
    pub struct UdevOutputId {
        pub device_id: DrmNode,
        pub crtc: crtc::Handle,
    }

    #[derive(Debug, thiserror::Error)]
    pub enum DeviceError {
        #[error("failed to create drm node: {0}")]
        DrmNode(#[from] CreateDrmNodeError),
        #[error("failed to open device via libseat: {0}")]
        LibSeat(#[from] libseat::Error),
        #[error("drm device error: {0}")]
        DrmDevice(#[from] DrmError),
        #[error("failed to setup gbm device: {0}")]
        GbmDevice(std::io::Error),
        #[error("failed to setup egl: {0}")]
        EGLError(#[from] egl::Error),
        #[error("failed to get render node for current egl")]
        EGLRenderNode,
        #[error("device {0} untracked")]
        DeviceUntracked(DrmNode),
        #[error("failed to scan connectors: {0}")]
        ScanConnector(std::io::Error),
        #[error("failed to query drm driver: {0}")]
        DrmDriver(std::io::Error),
        #[error("failed to setup drm output")]
        DrmOutput(String/* the error type is complicated */),
    }

    impl DeviceError {
        fn drm_output(error: impl std::error::Error) -> Self {
            Self::DrmOutput(error.to_string())
        }
    }
}

pub mod callbacks {
    //! handlers for various smithay event sources handler
    use super::*;

    type IoDisplay = NoIoDrop<Display<Trayle>>;
    type IoPostAction = std::io::Result<PostAction>;

    /// handler for [`Generic<Display>`] event source
    pub fn display<R>(_: R, display: &mut IoDisplay, trayle: &mut Trayle) -> IoPostAction {
        // SAFETY: we dont drop the display
        unsafe { display.get_mut().dispatch_clients(trayle).unwrap() };
        Ok(PostAction::Continue)
    }

    /// handler for [`LibinputInputBackend`] event source
    pub fn input(mut event: InputEvent, _: &mut (), trayle: &mut Trayle) {
        use smithay::reexports::input::DeviceCapability;

        match &mut event {
            InputEvent::DeviceAdded { device } => {
                if device.has_capability(DeviceCapability::Keyboard) {
                    if let Some(keyboard) = trayle.seat.get_keyboard() {
                        device.led_update(keyboard.led_state().into());
                    };
                    trayle.backend.keyboards.push(device.clone());
                }
            }
            InputEvent::DeviceRemoved { device } => {
                if device.has_capability(DeviceCapability::Keyboard) {
                    trayle.backend.keyboards.retain(|kb|kb!=device);
                }
            }
            _ => {}
        }

        input::handle(event, trayle);
    }

    /// handler for [`ListeningSocketSource`] event source
    ///
    /// [`ListeningSocketSource`]: smithay::wayland::socket::ListeningSocketSource
    pub fn socket(stream: UnixStream, _: &mut (), trayle: &mut Trayle) {
        let state = Arc::new(ClientState::default());
        match trayle.dh.insert_client(stream, state) {
            Ok(_client) => {}
            Err(err) => tracing::warn!("failed to add wayland client: {err}"),
        };
    }

    /// handler for [`LibSeatSessionNotifier`] event source
    ///
    /// [`LibSeatSessionNotifier`]: smithay::backend::session::libseat::LibSeatSessionNotifier
    pub fn session(event: SessionEvent, _: &mut (), trayle: &mut Trayle) {
        match event {
            SessionEvent::PauseSession => {
                tracing::info!("session pause");
                trayle.backend.input.suspend();
                for backend in trayle.backend.devices.values_mut() {
                    backend.drm_output_manager.pause();
                    backend.active_leases.clear();
                    if let Some(lease_global) = backend.drm_lease_state.as_mut() {
                        lease_global.suspend();
                    }
                }
            }
            SessionEvent::ActivateSession => {
                tracing::info!("session resume");
                if let Err(err) = trayle.backend.input.resume() {
                    tracing::error!("failed to resume libinput context: {err:?}");
                }
                for (&node, backend) in trayle.backend.devices.iter_mut() {
                    // if we do not care about flicking (caused by modesetting) we could just
                    // pass true for disable connectors here. this would make sure our drm
                    // device is in a known state (all connectors and planes disabled).
                    // but for demonstration we choose a more optimistic path by leaving the
                    // state as is and assume it will just work. If this assumption fails
                    // we will try to reset the state when trying to queue a frame.
                    backend.drm_output_manager
                        .activate(false /* disable connectors */)
                        .expect("failed to activate drm backend");
                    if let Some(lease_global) = backend.drm_lease_state.as_mut() {
                        lease_global.resume::<Trayle>();
                    }
                    trayle.handle.insert_idle(move|trayle|render::node(node, None, trayle.clock.now(), trayle));
                }
            }
        }
    }

    /// handler for [`DrmDeviceNotifier`] event source
    pub fn drm(event: DrmEvent, node: DrmNode, meta: &mut Option<DrmEventMetadata>, trayle: &mut Trayle) {
        match event {
            DrmEvent::VBlank(crtc) => {
                render::frame_finish(node, crtc, meta, trayle);
            }
            DrmEvent::Error(error) => {
                tracing::error!("{error:?}");
            }
        }
    }
}


pub mod render {
    use device::UdevOutputId;
    use crate::backend::UdevRenderer;
    use super::*;

    pub fn node(
        node: DrmNode,
        crtc: Option<crtc::Handle>,
        frame_target: Time<Monotonic>,
        trayle: &mut Trayle
    ) {
        let device_backend = match trayle.backend.devices.get_mut(&node) {
            Some(backend) => backend,
            None => {
                tracing::error!("trying to render on non-existent backend {node}");
                return;
            }
        };

        match crtc {
            Some(crtc) => {
                self::surface(node, crtc, frame_target, trayle);
            },
            None => {
                let crtcs = device_backend.surfaces.keys().copied().collect::<Vec<_>>();
                for crtc in crtcs {
                    self::surface(node, crtc, frame_target, trayle);
                }
            },
        }
    }

    pub fn surface(
        node: DrmNode,
        crtc: crtc::Handle,
        frame_target: Time<Monotonic>,
        trayle: &mut Trayle
    ) {
        // let Some(device) = trayle.backend.devices.get_mut(&node) else {
        //     return; //Err(DeviceError::DeviceUntracked(node));
        // };

        let Some(output) = trayle.frontend
            .space
            .outputs()
            .find(|o|{
                o.user_data().get::<UdevOutputId>()==Some(&UdevOutputId { device_id: node, crtc })
            })
            .cloned()
        else {
            // somehow called with invalid output
            return;
        };

        // self.pre_repaint(&output, frame_target);

        let Some(device) = trayle.backend.devices.get_mut(&node) else {
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
        let primary_gpu = trayle.backend.primary_gpu;
        let mut renderer = if primary_gpu == render_node {
            trayle.backend.gpus.single_renderer(&render_node)
        } else {
            let format = surface.drm_output.format();
            trayle.backend.gpus.renderer(&primary_gpu, &render_node, format)
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
            &trayle.frontend.space,
            &output,
            &trayle.config
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
                next_frame_target).saturating_sub(trayle.clock.now().into()
            );

            tracing::trace!("reschedule repaint timer with delay {reschedule_timeout:?} on {crtc:?}");

            let timer = Timer::from_duration(reschedule_timeout);
            trayle.handle.insert_source(timer, move|_,_,trayle|{
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

    pub fn frame_finish(node: DrmNode, crtc: crtc::Handle, meta: &mut Option<DrmEventMetadata>, trayle: &mut Trayle) {
        let device_backend = match trayle.backend.devices.get_mut(&node) {
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

        let Some(output) = trayle.frontend.space.outputs().find(|o|{
            o.user_data().get::<UdevOutputId>() == Some(&UdevOutputId {
                device_id: surface.device_id, crtc
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
            None => (trayle.clock.now(),
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

            let timer = if trayle.backend.primary_gpu != surface.render_node {
                // However, if we need to do a copy, that might not be enough.
                // (And without actual comparision to previous frames we cannot really know.)
                // So lets ignore that in those cases to avoid thrashing performance.
                tracing::trace!("scheduling repaint timer immediately on {crtc:?}");
                Timer::immediate()
            } else {
                tracing::trace!("scheduling repaint timer with delay {repaint_delay:?} on {crtc:?}");
                Timer::from_duration(repaint_delay)
            };

            trayle.handle.insert_source(timer, move|_,_,trayle|{
                render::node(node, Some(crtc), next_frame_target, trayle);
                TimeoutAction::Drop
            }).expect("failed to schedule frame timer");
        }
    }

    pub mod elements {
        use smithay::backend::renderer::Renderer;

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

pub mod input {
    use smithay::{backend::input::{Event, KeyboardKeyEvent}, input::keyboard::{FilterResult, KeysymHandle, ModifiersState}, utils::SERIAL_COUNTER};
    use xkbcommon::xkb::Keysym;

    use super::*;

    fn on_keyboard(trayle: &mut Trayle, mods: &ModifiersState, handle: KeysymHandle) -> FilterResult<()> {
        let keysym = handle.modified_sym();
        tracing::debug!(?mods, keysym = ::xkbcommon::xkb::keysym_get_name(keysym), "keysym");

        match keysym {
            Keysym::Return if mods.logo => {
                std::process::Command::new("alacritty")
                    .env("WAYLAND_DISPLAY", &trayle.frontend.wlsocket)
                    .spawn().inspect_err(|err|tracing::error!("{err}")).ok();
                FilterResult::Intercept(())
            }
            Keysym::Q if mods.logo => {
                tracing::info!("shutting down");
                trayle.signal.stop();
                // self.running.store(false, std::sync::atomic::Ordering::SeqCst);
                FilterResult::Intercept(())
            }
            _ => FilterResult::Forward
        }
    }
    pub fn handle(event: InputEvent, trayle: &mut Trayle) {
        if let InputEvent::Keyboard { event } = event {
            let serial = SERIAL_COUNTER.next_serial();
            let time = event.time_msec();
            trayle.seat.get_keyboard().unwrap().input::<(), _>(
                trayle,
                event.key_code(),
                event.state(),
                serial,
                time,
                self::on_keyboard,
            );
            return;
        }

        // match event {
        //     InputEvent::DeviceAdded { device } => todo!(),
        //     InputEvent::DeviceRemoved { device } => todo!(),
        //     InputEvent::Keyboard { event } => todo!(),
        //     InputEvent::PointerMotion { event } => todo!(),
        //     InputEvent::PointerMotionAbsolute { event } => todo!(),
        //     InputEvent::PointerButton { event } => todo!(),
        //     InputEvent::PointerAxis { event } => todo!(),
        //     InputEvent::GestureSwipeBegin { event } => todo!(),
        //     InputEvent::GestureSwipeUpdate { event } => todo!(),
        //     InputEvent::GestureSwipeEnd { event } => todo!(),
        //     InputEvent::GesturePinchBegin { event } => todo!(),
        //     InputEvent::GesturePinchUpdate { event } => todo!(),
        //     InputEvent::GesturePinchEnd { event } => todo!(),
        //     InputEvent::GestureHoldBegin { event } => todo!(),
        //     InputEvent::GestureHoldEnd { event } => todo!(),
        //     InputEvent::TouchDown { event } => todo!(),
        //     InputEvent::TouchMotion { event } => todo!(),
        //     InputEvent::TouchUp { event } => todo!(),
        //     InputEvent::TouchCancel { event } => todo!(),
        //     InputEvent::TouchFrame { event } => todo!(),
        //     InputEvent::TabletToolAxis { event } => todo!(),
        //     InputEvent::TabletToolProximity { event } => todo!(),
        //     InputEvent::TabletToolTip { event } => todo!(),
        //     InputEvent::TabletToolButton { event } => todo!(),
        //     InputEvent::SwitchToggle { event } => todo!(),
        //     InputEvent::Special(_) => todo!(),
        // }
    }
}



/// utilities for combining multiple domain logic
pub mod utils {
    use super::*;

    pub fn get_surface_dmabuf_feedback(
        primary_gpu: DrmNode,
        render_node: DrmNode,
        gpus: &mut GpuManager<GbmGlesBackend<GlesRenderer, DrmDeviceFd>>,
        surface: &smithay::backend::drm::DrmSurface,
    ) -> Option<SurfaceDmabufFeedback> {
        let primary_formats = gpus.single_renderer(&primary_gpu).ok()?.dmabuf_formats();
        let render_formats = gpus.single_renderer(&render_node).ok()?.dmabuf_formats();

        let all_render_formats = primary_formats
            .iter()
            .chain(render_formats.iter())
            .copied()
            .collect::<FormatSet>();

        let planes = surface.planes().clone();

        // limit the scanout tranche to formats that can also be render from
        // so that there is always a fallback render path available in case
        // the supplied buffer can not be scanned out directly
        let planes_formats = surface
            .plane_info()
            .formats
            .iter()
            .copied()
            .chain(planes.overlay.into_iter().flat_map(|p|p.formats))
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
                    planes_formats
                )
                .add_preference_tranche(render_node.dev_id(), None, render_formats)
                .build()
                .unwrap();

        Some(SurfaceDmabufFeedback {
            render_feedback,
            scanout_feedback
        })
    }
}

