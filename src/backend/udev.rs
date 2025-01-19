use std::{collections::HashMap, path::Path};

use drm_scanner::{display_info, DrmScanEvent, DrmScanner};
use smithay::{
    backend::{
        allocator::{
            format::FormatSet, gbm::{GbmAllocator, GbmBufferFlags, GbmDevice}, Fourcc
        },
        drm::{
            output::{DrmOutput, DrmOutputManager, DrmOutputRenderElements}, DrmDevice, DrmDeviceFd, DrmEvent, DrmEventMetadata, DrmNode, DrmSurface, NodeType
        },
        egl::{context::ContextPriority, EGLDevice, EGLDisplay},
        input::InputEvent,
        libinput::{LibinputInputBackend, LibinputSessionInterface},
        renderer::{
            element::{utils::{CropRenderElement, RelocateRenderElement, RescaleRenderElement}, Wrap}, gles::GlesRenderer, multigpu::{gbm::GbmGlesBackend, GpuManager}, ImportAll, ImportDma, ImportMem, ImportMemWl
        },
        session::{self, libseat::LibSeatSession, Session},
        udev::{self, all_gpus, primary_gpu, UdevBackend},
    }, desktop::{space::SpaceRenderElements, utils::OutputPresentationFeedback}, output::{Mode as WlMode, Output, PhysicalProperties}, reexports::{
        calloop::{EventLoop, LoopHandle, RegistrationToken}, drm::control::{connector, crtc, ModeTypeFlags}, input::{DeviceCapability, Libinput}, rustix::fs::OFlags, wayland_protocols::wp::linux_dmabuf::zv1::server::zwp_linux_dmabuf_feedback_v1, wayland_server::{backend::GlobalId, Display, DisplayHandle}
    }, wayland::dmabuf::{DmabufFeedback, DmabufFeedbackBuilder, DmabufGlobal, DmabufState}
};

use crate::{shell::fixup_positions, state::BackendState, State};

struct UdevData {
    state: State,
    handle: LoopHandle<'static, UdevData>,
    dh: smithay::reexports::wayland_server::DisplayHandle,

    session: LibSeatSession,
    primary_gpu: DrmNode,
    gpus: GpuManager<GbmGlesBackend<GlesRenderer, DrmDeviceFd>>,
    backends: HashMap<DrmNode, BackendData>,
    dmabuf_state: Option<(DmabufState, DmabufGlobal)>,
}

impl BackendState for UdevData {
    fn state(&self) -> &State {
        &self.state
    }

    fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }
}

struct BackendData {
    registration_token: RegistrationToken,
    drm_output_manager: DrmOutputManager<
        GbmAllocator<DrmDeviceFd>,
        GbmDevice<DrmDeviceFd>,
        Option<OutputPresentationFeedback>,
        DrmDeviceFd,
    >,
    render_node: DrmNode,
    drm_scanner: DrmScanner,
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut event_loop = EventLoop::<UdevData>::try_new()?;
    let handle = event_loop.handle();
    let display = Display::<State>::new()?;
    let dh = display.handle();
    let state = State::new(&mut event_loop, display);

    // NOTE: compositor require to manage three main aspects of interaction with the OS
    // - session management
    // - input handling
    // - graphics

    //
    // NOTE: session management, via `libseat`
    //
    let (session, session_notifier) = LibSeatSession::new()?;
    let seat = session.seat();

    let primary_gpu = primary_gpu(&seat)
        .unwrap()
        .and_then(|x|DrmNode::from_path(x).ok()?.node_with_type(NodeType::Render)?.ok())
        .unwrap_or_else(||{
            all_gpus(&seat)
                .unwrap()
                .into_iter()
                .find_map(|x|DrmNode::from_path(x).ok())
                .expect("No GPU!")
        });
    let gpus = GpuManager::new(GbmGlesBackend::with_context_priority(ContextPriority::High)).unwrap();

    tracing::debug!("using gpu: {primary_gpu}");

    let mut data = UdevData {
        state,
        handle,
        dh,
        session,
        primary_gpu,
        gpus,
        backends: HashMap::new(),
        dmabuf_state: None,
    };

    let udev_backend = UdevBackend::new(&seat)?;

    for (device_id, path) in udev_backend.device_list() {
        if let Ok(node) = DrmNode::from_dev_id(device_id) {
            if let Err(err) = data.device_added(node, path) {
                tracing::error!("Device {device_id} error: {err}, skipping");
            }
        };
    }
    data.state.shm_state.update_formats(data.gpus.single_renderer(&primary_gpu).unwrap().shm_formats());

    //
    // NOTE: input handling, via `libinput`
    //
    let mut libinput = Libinput::new_with_udev::<LibinputSessionInterface<LibSeatSession>>(
        data.session.clone().into(),
    );
    libinput.udev_assign_seat(&seat).unwrap();
    let libinput_backend = LibinputInputBackend::new(libinput.clone());


    #[allow(unused_mut)]
    let mut renderer = data.gpus.single_renderer(&primary_gpu).unwrap();

    // egl
    // renderer.bind_wl_display

    // init dmabuf support
    let dmabuf_format = renderer.dmabuf_formats();
    let default_feedback = DmabufFeedbackBuilder::new(primary_gpu.dev_id(), dmabuf_format)
        .build()
        .unwrap();
    let mut dmabuf_state = DmabufState::new();
    let global = dmabuf_state
        .create_global_with_default_feedback::<State>(&data.state.display_handle, &default_feedback);
    data.dmabuf_state = Some((dmabuf_state, global));

    // for (_, _backend) in &mut backends {
    //     // LATEST: last boss
    //     // backend.surfaces;
    // }

    //
    // register all the event sources
    //

    event_loop.handle().insert_source(libinput_backend, |mut event, _, data|{
        match &mut event {
            InputEvent::DeviceAdded { device } => {
                if device.has_capability(DeviceCapability::Keyboard) {
                    tracing::debug!("[APP] keyboard connected");
                }
            }
            InputEvent::DeviceRemoved { device } => {
                if device.has_capability(DeviceCapability::Keyboard) {
                    tracing::debug!("[APP] keyboard disconnected");
                }
            }
            // InputEvent::Keyboard { event } => {}
            // InputEvent::PointerMotion { event } => {}
            // InputEvent::PointerMotionAbsolute { event } => {}
            // InputEvent::PointerButton { event } => {}
            // InputEvent::PointerAxis { event } => {}
            _ => {}
        }

        data.state.process_input_event(event);
    })?;

    event_loop.handle().insert_source(session_notifier, move|event, _, data|{
        match event {
            session::Event::PauseSession => {
                libinput.suspend();
                for (_, backend) in &mut data.backends {
                    backend.drm_output_manager.pause();
                    // backend.active_leases.clear
                    // backend.leasing_global.suspend
                }
            }
            session::Event::ActivateSession => {
                libinput.resume().expect("failed to resume libinput");
                for (_, backend) in &mut data.backends {
                    backend
                        .drm_output_manager
                        .activate(false)
                        .expect("failed to activate drm backend");
                    // backend.leasing_global.suspend
                    // event_loop.handle().insert_idle(move|data|data.render);
                }
            }
        }
    })?;

    event_loop.handle().insert_source(udev_backend, |event, _, _|{
        // track devices internally
        tracing::debug!("udev event! {event:?}");
        match event {
            udev::UdevEvent::Added { device_id: _, path: _ } => {}
            udev::UdevEvent::Changed { device_id: _ } => {}
            udev::UdevEvent::Removed { device_id: _ } => {}
        }
    })?;

    event_loop.run(None, &mut data, |_|{})?;

    std::env::set_var("WAYLAND_DISPLAY", &data.state.socket_name);
    tracing::debug!("wayland socket: {:?}",&data.state.socket_name);

    //
    // post setup
    //

    data.state.open_terminal();

    Ok(())
}

impl UdevData {
    fn device_added(&mut self, node: DrmNode, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let fd = self.session
            .open(path, OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOCTTY | OFlags::NONBLOCK)?;

        let fd = DrmDeviceFd::new(fd.into());
        let (drm, notifier) = DrmDevice::new(fd.clone(), true)?;
        let gbm = GbmDevice::new(fd)?;

        let registration_token = self.handle.insert_source(notifier, move|event, meta, data|{
            match event {
                DrmEvent::VBlank(crtc) => {
                    data.frame_finish(node, crtc, meta);
                }
                DrmEvent::Error(error) => {
                    tracing::error!("[DRM] {error}");
                }
            }
        })?;

        // egl
        let render_node = EGLDevice::device_for_display(&unsafe { EGLDisplay::new(gbm.clone())? })
            .ok()
            .and_then(|x| x.try_get_render_node().ok().flatten())
            .unwrap_or(node);

        self.gpus.as_mut().add_node(render_node, gbm.clone())?;

        const SUPPORTED_FORMATS: &[Fourcc] = &[
            Fourcc::Abgr2101010,
            Fourcc::Argb2101010,
            Fourcc::Abgr8888,
            Fourcc::Argb8888,
        ];

        let allocator = GbmAllocator::new(gbm.clone(), GbmBufferFlags::RENDERING | GbmBufferFlags::SCANOUT);
        let color_formats = SUPPORTED_FORMATS;
        let mut renderer = self.gpus.single_renderer(&render_node)?;
        let renderer_formats = renderer.as_mut().egl_context().dmabuf_render_formats().clone();

        let drm_device_manager = DrmOutputManager::new(
            drm, allocator, gbm.clone(), Some(gbm), color_formats.iter().copied(), renderer_formats
        );

        self.backends.insert(node, BackendData {
            registration_token,
            drm_output_manager: drm_device_manager,
            render_node,

            drm_scanner: DrmScanner::default(),
            // surfaces: Hash
        });

        if let Err(err) = self.device_changed(node) {
            tracing::error!("{err}");
        };

        Ok(())
    }

    fn device_changed(
        &mut self,
        node: DrmNode,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(device) = self.backends.get_mut(&node) else {
            return Ok(());
        };

        let scan_result = device
            .drm_scanner
            .scan_connectors(device.drm_output_manager.device())?;

        for event in scan_result {
            match event {
                DrmScanEvent::Connected {
                    connector,
                    crtc: Some(crtc),
                } => {
                    self.connector_connected(connector, crtc, node);
                }
                DrmScanEvent::Disconnected {
                    connector: _,
                    crtc: Some(crtc),
                } => {
                    // connector_disconnected(backends, node);
                }
                _ => {}
            }
        }

        fixup_positions();

        Ok(())
    }

    fn connector_connected(
        &mut self,
        connector: connector::Info,
        crtc: crtc::Handle,
        node: DrmNode,
    ) {
        let Some(device) = self.backends.get_mut(&node) else {
            return;
        };

        let mut renderer = self.gpus.single_renderer(&device.render_node).unwrap();

        // debug
        let output_name = format!("{}-{}", connector.interface().as_str(), connector.interface_id());
        tracing::info!(?crtc,"try to setup connector: {output_name}");

        let drm_device = device.drm_output_manager.device();
        use smithay::reexports::drm::control::Device as ControlDevice;
        let non_desktop = drm_device
            .get_properties(connector.handle())
            .ok()
            .and_then(|props|{
                let (info, value) = props.into_iter()
                    .filter_map(|(handle, value)|{
                        let info = drm_device.get_property(handle).ok()?;
                        Some((info,value))
                    })
                    .find(|(info,_)|info.name().to_str() == Ok("non-desktop"))?;

                info.value_type().convert_value(value).as_boolean()
            })
            .unwrap_or(false);

        let display_info = display_info::for_connector(drm_device, connector.handle());

        let make = display_info
            .as_ref()
            .and_then(|info|info.make())
            .unwrap_or_else(||"Unknown".into());

        let model = display_info
            .as_ref()
            .and_then(|info|info.model())
            .unwrap_or_else(||"Unknown".into());

        if non_desktop {
            tracing::warn!("there is no non-desktop here");
            return;
        }

        let mode_id = connector
            .modes()
            .iter()
            .position(|mode|mode.mode_type().contains(ModeTypeFlags::PREFERRED))
            .unwrap_or(0);

        let drm_mode = connector.modes()[mode_id];
        let wl_mode = WlMode::from(drm_mode);

        let (phys_w, phys_h) = connector.size().unwrap_or((0,0));
        let output = Output::new(output_name, PhysicalProperties {
            size: (phys_w as i32, phys_h as i32).into(),
            subpixel: connector.subpixel().into(),
            make,
            model,
        });

        let global = output.create_global::<State>(&self.dh);

        let x = self
            .state
            .space
            .outputs()
            .fold(0, |acc, o|acc + self.state.space.output_geometry(o).unwrap().size.w);
        let position = (x, 0).into();

        output.set_preferred(wl_mode);
        output.change_current_state(Some(wl_mode), None, None, Some(position));
        self.state.space.map_output(&output, position);

        output.user_data().insert_if_missing(||UdevOutputId {
            device_id: node,
            crtc,
        });

        use smithay::reexports::drm::Device;
        let Ok(driver) = drm_device.get_driver() else {
            tracing::warn!("Failed to query drm driver");
            return;
        };

        let Ok(mut planes) = drm_device.planes(&crtc) else {
            tracing::warn!("Failed to query crtc planes");
            return;
        };

        if driver.name().to_string_lossy().to_lowercase().contains("nvidia")
            || driver.description().to_string_lossy().to_lowercase().contains("nvidia")
        {
            planes.overlay = vec![];
        }

        /*
        let result = device.drm_output_manager.initialize_output::<_, OutputRenderElements>(
            crtc,
            drm_mode,
            &[connector.handle()],
            &output,
            Some(planes),
            &mut renderer,
            &DrmOutputRenderElements::default(), // LATEST:
        );

        let drm_output = match result {
            Ok(ok) => ok,
            Err(err) => {
                tracing::error!("{err}");
                return;
            },
        };

        let disable_direct_scanout = false;

        let dmabuf_feedback = drm_output.with_compositor(|compositor|{
            get_surface_dmabuf_feedback(
                self.primary_gpu,
                device.render_node,
                &mut self.gpus,
                compositor.surface(),
            )
        });

        let surface_data = SurfaceData {
            dh: self.dh.clone(),
            device_id: node,
            render_node: device.render_node,
            global: Some(global),
            drm_output,
            disable_direct_scanout,
            dmabuf_feedback,
        };
        */

        todo!("following anvil example still lot of abstraction")
    }

    fn connector_disconnected(&mut self, node: DrmNode) {
        todo!()
    }

    fn frame_finish(
        &mut self,
        _node: DrmNode,
        _handle: crtc::Handle,
        _meta: &mut Option<DrmEventMetadata>)
    {
        todo!()
    }
}

fn get_surface_dmabuf_feedback(
    primary_gpu: DrmNode,
    render_node: DrmNode,
    gpus: &mut GpuManager<GbmGlesBackend<GlesRenderer, DrmDeviceFd>>,
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
    let planes_format = surface
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
            planes_format,
        )
        .add_preference_tranche(render_node.dev_id(), None, render_formats)
        .build()
        .unwrap();

    Some(SurfaceDmabufFeedback {
        render_feedback,
        scanout_feedback
    })
}

struct UdevOutputId {
    device_id: DrmNode,
    crtc: crtc::Handle,
}

struct SurfaceDmabufFeedback {
    pub render_feedback: DmabufFeedback,
    pub scanout_feedback: DmabufFeedback,
}

struct SurfaceData {
    dh: DisplayHandle,
    device_id: DrmNode,
    render_node: DrmNode,
    global: Option<GlobalId>,
    drm_output: DrmOutput<
        GbmAllocator<DrmDeviceFd>,
        GbmDevice<DrmDeviceFd>,
        Option<OutputPresentationFeedback>,
        DrmDeviceFd
    >,
    disable_direct_scanout: bool,
    dmabuf_feedback: Option<SurfaceDmabufFeedback>
}

