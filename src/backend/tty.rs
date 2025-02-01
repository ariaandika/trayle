use std::{
    collections::HashMap,
    sync::{atomic::Ordering, Mutex},
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use smithay::{
    backend::{
        allocator::{
            format::FormatSet,
            gbm::{GbmAllocator, GbmBufferFlags, GbmDevice},
            Fourcc,
        },
        drm::{
            compositor::{FrameFlags, RenderFrameError},
            output::{DrmOutput, DrmOutputManager, DrmOutputRenderElements},
            DrmAccessError, DrmDevice, DrmDeviceFd, DrmError, DrmEvent, DrmEventMetadata, DrmNode,
            NodeType,
        },
        egl::{context::ContextPriority, EGLDevice, EGLDisplay},
        input::InputEvent,
        libinput::{LibinputInputBackend, LibinputSessionInterface},
        renderer::{
            damage::Error as OutputDamageTrackerError,
            element::{memory::MemoryRenderBuffer, AsRenderElements, RenderElementStates},
            gles::GlesRenderer,
            multigpu::{gbm::GbmGlesBackend, GpuManager, MultiRenderer},
            DebugFlags, ImportDma, ImportEgl, ImportMemWl,
        },
        session::{libseat::LibSeatSession, Event as SessionEvent, Session},
        udev::{self, UdevBackend, UdevEvent},
        SwapBuffersError,
    },
    desktop::{space::SurfaceTree, utils::OutputPresentationFeedback, Space},
    input::pointer::{CursorImageAttributes, CursorImageStatus},
    output::{Mode as WlMode, Output, PhysicalProperties},
    reexports::{
        calloop::{
            timer::{TimeoutAction, Timer},
            EventLoop, LoopHandle, RegistrationToken,
        },
        drm::{
            control::{connector, crtc, Device as ControlDevice, ModeTypeFlags},
            Device,
        },
        input::{DeviceCapability, Libinput},
        rustix::fs::OFlags,
        wayland_protocols::wp::{
            linux_dmabuf::zv1::server::zwp_linux_dmabuf_feedback_v1,
            presentation_time::server::wp_presentation_feedback,
        },
        wayland_server::{backend::GlobalId, Display, DisplayHandle},
    },
    utils::{IsAlive, Logical, Monotonic, Point, Scale, Time, Transform},
    wayland::{
        compositor,
        dmabuf::{DmabufFeedbackBuilder, DmabufGlobal, DmabufState},
        drm_lease::{DrmLease, DrmLeaseState},
        drm_syncobj::DrmSyncobjState,
        presentation::Refresh,
    },
};

use crate::{
    drawing::PointerElement,
    render::{output_elements, CustomRenderElements, OutputRenderElements},
    shell::elements::{WindowElement, WindowRenderElement},
    state::{BackendState, DndIcon, SurfaceDmabufFeedback},
    utils::{
        display_info,
        drm_scanner::{DrmScanEvent, DrmScanner},
    },
};

mod drm_lease;
mod drm_syncobj;

type Trayle = crate::Trayle<Tty>;

type UdevRenderer<'a> = MultiRenderer<
    'a,'a,
    GbmGlesBackend<GlesRenderer, DrmDeviceFd>,
    GbmGlesBackend<GlesRenderer, DrmDeviceFd>,
>;

const SUPPORTED_FORMATS_8BIT_ONLY: &[Fourcc] = &[Fourcc::Abgr8888, Fourcc::Argb8888];

const SUPPORTED_FORMATS: &[Fourcc] = &[
    Fourcc::Abgr2101010,
    Fourcc::Argb2101010,
    Fourcc::Abgr8888,
    Fourcc::Argb8888,
];

struct Tty {
    dh: DisplayHandle,
    loop_handle: LoopHandle<'static, Trayle>,
    seat: String,
    session: LibSeatSession,
    dmabuf_state: Option<(DmabufState, DmabufGlobal)>,
    syncobj_state: Option<DrmSyncobjState>,
    primary_gpu: DrmNode,
    gpus: GpuManager<GbmGlesBackend<GlesRenderer,DrmDeviceFd>>,
    backends: HashMap<DrmNode, BackendData>,
    pointer_image: crate::cursor::Cursor,
    pointer_images: Vec<(xcursor::parser::Image,MemoryRenderBuffer)>,
    pointer_element: PointerElement,
    debug_flags: DebugFlags,
    keyboards: Vec<smithay::reexports::input::Device>,
}

impl BackendState for Tty {
    fn seat(&self) -> &str {
        &self.seat
    }
    fn dh(&self) -> &DisplayHandle {
        &self.dh
    }
    fn dh_mut(&mut self) -> &mut DisplayHandle {
        &mut self.dh
    }
    fn loop_handle(&mut self) -> &mut LoopHandle<'static, Trayle> {
        &mut self.loop_handle
    }
}

pub fn run() -> Result<()> {
    let mut event_loop = EventLoop::<Trayle>::try_new().context("failed to setup eventloop")?;
    let display = Display::<Trayle>::new().context("failed to setup wayland display")?;
    let (session,session_source) = LibSeatSession::new().context("failed to setup libseat session")?;

    let loop_handle = event_loop.handle();
    let dh = display.handle();
    let seat = session.seat();

    // setup gpu devices
    let primary_gpu = udev::primary_gpu(&seat)
        .context("failed to query gpu")?
        .and_then(|gpu|DrmNode::from_path(gpu).ok()?.node_with_type(NodeType::Render)?.ok());
    let primary_gpu = match primary_gpu {
        Some(ok) => ok,
        None => udev::all_gpus(&seat)
            .context("failed to query gpu")?
            .into_iter()
            .find_map(|gpu|DrmNode::from_path(gpu).ok())
            .context("no gpu found")?,
    };
    let graphics_api = GbmGlesBackend::with_context_priority(ContextPriority::High);
    let gpus = GpuManager::new(graphics_api).context("failed to setup gbm gles renderer")?;
    tracing::info!("using {primary_gpu} as primary gpu");


    let backend = Tty {
        dh,
        loop_handle,
        seat,
        session,
        primary_gpu,
        dmabuf_state: None,
        syncobj_state: None,
        gpus,
        backends: HashMap::new(),
        pointer_image: crate::cursor::Cursor::load(),
        pointer_images: vec![],
        pointer_element: PointerElement::default(),
        debug_flags: DebugFlags::empty(),
        keyboards: vec![],
    };
    let mut trayle = Trayle::setup(&mut event_loop, display, backend)?;


    // setup udev
    let udev_backend = UdevBackend::new(&trayle.backend.seat).context("failed to setup udev backend")?;
    for (device_id,path) in udev_backend.device_list() {
        Trayle::handle_udev(UdevEvent::Added { device_id, path: path.to_path_buf() }, &mut (), &mut trayle);
    }
    trayle.backend.loop_handle.insert_source(udev_backend, Trayle::handle_udev).unwrap();


    // setup libinput
    let session = trayle.backend.session.clone();
    let mut libinput_context = Libinput::new_with_udev::<LibinputSessionInterface<LibSeatSession>>(session.into());
    libinput_context.udev_assign_seat(&trayle.backend.seat).unwrap();
    let libinput_source = LibinputInputBackend::new(libinput_context.clone());

    trayle.backend.loop_handle.insert_source(libinput_source, |mut event,_,data|{
        match &mut event {
            InputEvent::DeviceAdded { device } => {
                if device.has_capability(DeviceCapability::Keyboard) {
                    if let Some(led_state) = data.seat.get_keyboard().map(|kb|kb.led_state()) {
                        device.led_update(led_state.into());
                    };
                    data.backend.keyboards.push(device.clone());
                }
            }
            InputEvent::DeviceRemoved { device } => {
                if device.has_capability(DeviceCapability::Keyboard) {
                    data.backend.keyboards.retain(|kb|kb!=device);
                }
            }
            _ => {}
        }

        data.process_input_event(event);
    }).unwrap();

    trayle.backend.loop_handle.insert_source(session_source, move|event, _, data| match event {
        SessionEvent::PauseSession => {
            tracing::info!("session pause");
            libinput_context.suspend();
            for backend in data.backend.backends.values_mut() {
                backend.drm_output_manager.pause();
                backend.active_leases.clear();
                if let Some(lease_global) = backend.leasing_global.as_mut() {
                    lease_global.suspend();
                }
            }
        }
        SessionEvent::ActivateSession => {
            tracing::info!("session resume");
            if let Err(err) = libinput_context.resume() {
                tracing::error!("failed to resume libinput context: {err:?}");
            }
            for (&node, backend) in data.backend.backends.iter_mut() {
                // if we do not care about flicking (caused by modesetting) we could just
                // pass true for disable connectors here. this would make sure our drm
                // device is in a known state (all connectors and planes disabled).
                // but for demonstration we choose a more optimistic path by leaving the
                // state as is and assume it will just work. If this assumption fails
                // we will try to reset the state when trying to queue a frame.
                backend.drm_output_manager
                    .activate(false /* disable connectors */)
                    .expect("failed to activate drm backend");
                if let Some(lease_global) = backend.leasing_global.as_mut() {
                    lease_global.resume::<Trayle>();
                }
                data.backend.loop_handle.insert_idle(move|data|{
                    data.render(node, None, data.clock.now());
                });
            }
        }
    }).unwrap();


    let mut renderer = trayle.backend.gpus.single_renderer(&primary_gpu).unwrap();

    // setup shared memory formats
    trayle.shm_state.update_formats(renderer.shm_formats());

    // try to enable EGL hardware-acceleration
    match renderer.bind_wl_display(&trayle.backend.dh) {
        Ok(_) => tracing::info!("EGL hardware-acceleration enabled"),
        Err(err) => tracing::info!(?err, "EGL hardware-acceleration disabled"),
    };

    // setup dmabuf support with format list from primary gpu
    let dmabuf_formats = renderer.dmabuf_formats();
    let default_feedback = DmabufFeedbackBuilder::new(primary_gpu.dev_id(), dmabuf_formats)
        .build()
        .unwrap();
    let mut dmabuf_state = DmabufState::new();
    let global = dmabuf_state.create_global_with_default_feedback::<Trayle>(
        &trayle.backend.dh, &default_feedback
    );
    trayle.backend.dmabuf_state = Some((dmabuf_state, global));

    // update each drm surface dmabuf feedback
    for backend_data in trayle.backend.backends.values_mut() {
        for surface_data in backend_data.surfaces.values_mut() {
            surface_data.dmabuf_feedback = surface_data.dmabuf_feedback.take().or_else(||{
                surface_data.drm_output.with_compositor(|compositor|{
                    get_surface_dmabuf_feedback(
                        primary_gpu,
                        surface_data.render_node,
                        &mut trayle.backend.gpus,
                        compositor.surface(),
                    )
                })
            });
        }
    }

    // expose syncobj protocol if supported by primary gpu
    if let Some(primary_node) = trayle
        .backend
        .primary_gpu
        .node_with_type(NodeType::Primary)
        .and_then(Result::ok)
    {
        if let Some(backend) = trayle.backend.backends.get(&primary_node) {
            let import_device = backend.drm_output_manager.device().device_fd().clone();

            if smithay::wayland::drm_syncobj::supports_syncobj_eventfd(&import_device) {
                let syncobj_state = DrmSyncobjState::new::<Trayle>(&trayle.backend.dh, import_device);
                trayle.backend.syncobj_state = Some(syncobj_state);
            }
        }
    }

    // start xwayland
    if matches!(std::env::var("TRAYLE_XWAYLAND").as_deref(),Ok("1")) {
        if let Err(err) = trayle.start_xwayland() {
            tracing::warn!("{err}");
        };
    }

    // showtime
    while trayle.running.load(Ordering::SeqCst) {
        let result = event_loop.dispatch(Some(Duration::from_millis(10)), &mut trayle);
        match result {
            Ok(_) => {
                trayle.space.refresh();
                // state.popups.refresh();
                trayle.backend.dh.flush_clients().unwrap();
            },
            Err(err) => {
                tracing::error!("loop error: {err:?}");
                trayle.running.store(false, Ordering::SeqCst);
            },
        }
    }

    tracing::info!("event loop exited");

    Ok(())
}

impl Trayle {
    fn handle_udev(event: UdevEvent, _: &mut (), data: &mut Trayle) {
        match event {
            UdevEvent::Added { device_id, path } => {
                match DrmNode::from_dev_id(device_id) {
                    Ok(node) => if let Err(err) = data.device_added(node, &path) {
                        tracing::error!("skipping device {device_id}: {err}");
                    },
                    Err(err) => {
                        tracing::error!("skipping device {device_id}: {err}");
                    },
                }
            }
            UdevEvent::Changed { device_id } => {
                if let Ok(node) = DrmNode::from_dev_id(device_id) {
                    data.device_changed(node);
                }
            }
            UdevEvent::Removed { device_id } => {
                if let Ok(node) = DrmNode::from_dev_id(device_id) {
                    data.device_removed(node);
                }
            }
        }
    }

    fn device_added(&mut self, node: DrmNode, path: &std::path::Path) -> Result<()> {
        let flags = OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOCTTY | OFlags::NONBLOCK;
        let fd = self.backend.session.open(path, flags).context("failed to open device")?;
        let fd = DrmDeviceFd::new(fd.into());

        let (drm,drm_source) = DrmDevice::new(fd.clone(), true).context("failed to setup drm device")?;
        let gbm = GbmDevice::new(fd).context("failed to setup gbm device")?;

        let token = self
            .backend
            .loop_handle
            .insert_source(drm_source, move|event,meta,data|match event {
                DrmEvent::VBlank(crtc) => {
                    data.frame_finish(node, crtc, meta);
                }
                DrmEvent::Error(error) => {
                    tracing::error!("{error:?}");
                }
            })
            .unwrap();


        let display = unsafe { EGLDisplay::new(gbm.clone()) }.context("failed to setup EGL display")?;

        let render_node = EGLDevice::device_for_display(&display)
            .ok()
            .and_then(|e| e.try_get_render_node().ok().flatten())
            .unwrap_or(node);

        self.backend
            .gpus
            .as_mut()
            .add_node(render_node, gbm.clone())
            .context("failed to add node to gpu manager")?;

        let color_formats = match std::env::var("TRAYLE_DISABLE_DIRECT_10BIT") {
            Ok(_) => SUPPORTED_FORMATS_8BIT_ONLY,
            Err(_) => SUPPORTED_FORMATS
        };
        let gbm_buffer_flags = GbmBufferFlags::RENDERING | GbmBufferFlags::SCANOUT;
        let allocator = GbmAllocator::new(gbm.clone(), gbm_buffer_flags);
        let mut renderer = self.backend.gpus.single_renderer(&render_node).unwrap();
        let render_formats = renderer.as_mut().egl_context().dmabuf_render_formats().clone();

        let drm_output_manager = DrmOutputManager::new(
            drm,
            allocator,
            gbm.clone(),
            Some(gbm),
            color_formats.iter().copied(),
            render_formats
        );

        let backend_data = BackendData {
            token,
            drm_output_manager,
            drm_scanner: DrmScanner::new(),
            non_desktop_connectors: vec![],
            render_node,
            surfaces: HashMap::new(),
            leasing_global: DrmLeaseState::new::<Trayle>(&self.backend.dh, &node)
                .inspect_err(|err|{
                    tracing::warn!(?err,"failed to setup drm lease global for: {node}");
                })
                .ok(),
            active_leases: vec![],
        };

        self.backend.backends.insert(node, backend_data);
        self.device_changed(node);

        Ok(())
    }

    fn device_changed(&mut self, node: DrmNode) {
        let Some(device) = self.backend.backends.get_mut(&node) else {
            return;
        };

        let scan_result = match device.drm_scanner.scan_connectors(device.drm_output_manager.device()) {
            Ok(ok) => ok,
            Err(err) => {
                tracing::warn!(?err, "failed to scan connectors");
                return;
            },
        };

        for event in scan_result {
            match event {
                DrmScanEvent::Connected {
                    connector,
                    crtc: Some(crtc),
                } => self.connector_connected(node, connector, crtc),
                DrmScanEvent::Disconnected {
                    connector,
                    crtc: Some(crtc),
                } => self.connector_disconnected(node, connector, crtc),
                _ => {}
            }
        }

        // fixup window coordinates
        crate::shell::utils::fixup_positions(&mut self.space, self.pointer.current_location());
    }

    fn device_removed(&mut self, node: DrmNode) {
        let Some(device) = self.backend.backends.get_mut(&node) else {
            return;
        };

        let crtcs = device
            .drm_scanner
            .crtcs()
            .map(|(info, crtc)| (info.clone(), crtc))
            .collect::<Vec<_>>();

        for (connector,crtc) in crtcs {
            self.connector_disconnected(node, connector, crtc);
        }

        tracing::debug!("surfaces dropped");

        if let Some(mut backend_data) = self.backend.backends.remove(&node) {
            if let Some(mut leasing_global) = backend_data.leasing_global.take() {
                leasing_global.disable_global::<Trayle>();
            }

            self.backend
                .gpus
                .as_mut()
                .remove_node(&backend_data.render_node);

            self.backend.loop_handle.remove(backend_data.token);
        }

        crate::shell::utils::fixup_positions(&mut self.space, self.pointer.current_location());
    }

    fn frame_finish(&mut self, dev_id: DrmNode, crtc: crtc::Handle, meta: &mut Option<DrmEventMetadata>) {
        let device_backend = match self.backend.backends.get_mut(&dev_id) {
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

        let Some(output) = self.space.outputs().find(|o|{
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
            None => (self.clock.now(),
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

            let timer = if self.backend.primary_gpu != surface.render_node {
                // However, if we need to do a copy, that might not be enough.
                // (And without actual comparision to previous frames we cannot really know.)
                // So lets ignore that in those cases to avoid thrashing performance.
                tracing::trace!("scheduling repaint timer immediately on {crtc:?}");
                Timer::immediate()
            } else {
                tracing::trace!("scheduling repaint timer with delay {repaint_delay:?} on {crtc:?}");
                Timer::from_duration(repaint_delay)
            };

            self.backend.loop_handle.insert_source(timer, move|_,_,data|{
                data.render(dev_id, Some(crtc), next_frame_target);
                TimeoutAction::Drop
            }).expect("failed to schedule frame timer");
        }
    }

    fn connector_connected(&mut self, node: DrmNode, connector: connector::Info, crtc: crtc::Handle) {
        let Some(device) = self.backend.backends.get_mut(&node) else {
            return;
        };

        let mut renderer = self
            .backend
            .gpus
            .single_renderer(&device.render_node)
            .unwrap();

        let output_name = format!("{}-{}",connector.interface().as_str(),connector.interface_id());
        tracing::info!(?crtc, "setting up connector {}",output_name);

        let drm_device = device.drm_output_manager.device();

        let non_desktop = drm_device.get_properties(connector.handle())
            .ok()
            .and_then(|props|{
                let (info,value) = props.into_iter().filter_map(|(handle,value)|{
                    let info = drm_device.get_property(handle).ok()?;
                    Some((info,value))
                })
                .find(|(info,_)|info.name().to_str()==Ok("non-desktop"))?;
                info.value_type().convert_value(value).as_boolean()
            })
            .unwrap_or(false);

        let display_info = display_info::for_connectors(drm_device, connector.handle());

        let make = display_info.as_ref().and_then(|info|info.make()).unwrap_or_else(||"Unknown".into());
        let model = display_info.as_ref().and_then(|info|info.model()).unwrap_or_else(||"Unknown".into());

        if non_desktop {
            tracing::info!("connector {} is non-desktop, setting up for leasing", output_name);
            device.non_desktop_connectors.push((connector.handle(), crtc));
            if let Some(lease_state) = device.leasing_global.as_mut() {
                lease_state.add_connector::<Trayle>(
                    connector.handle(),
                    output_name,
                    format!("{make} {model}")
                );
            }
            return;
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

        let global = output.create_global::<Trayle>(&self.backend.dh);

        let x = self.space.outputs().fold(0, |acc, o| {
            acc + self.space.output_geometry(o).unwrap().size.w
        });

        let position = (x, 0).into();

        output.set_preferred(wl_mode);
        output.change_current_state(Some(wl_mode), None, None, Some(position));
        self.space.map_output(&output, position);

        output.user_data().insert_if_missing(||UdevOutputId { crtc, device_id: node });

        let driver = match drm_device.get_driver() {
            Ok(driver) => driver,
            Err(err) => {
                tracing::warn!("failed to query drm driver: {err}");
                return;
            },
        };

        let mut planes = match drm_device.planes(&crtc) {
            Ok(planes) => planes,
            Err(err) => {
                tracing::warn!("failed to query crtc planes: {err}");
                return;
            },
        };

        if driver.name().to_string_lossy().to_lowercase().contains("nvidia") ||
            driver.description().to_string_lossy().to_lowercase().contains("nvidia")
        {
            planes.overlay.clear();
        }

        let result = device.drm_output_manager
            .initialize_output::<_, OutputRenderElements<UdevRenderer<'_>, WindowRenderElement<UdevRenderer<'_>>>>(
                crtc,
                drm_mode,
                &[connector.handle()],
                &output,
                Some(planes),
                &mut renderer,
                &DrmOutputRenderElements::default()
            );

        let drm_output = match result {
            Ok(ok) => ok,
            Err(err) => {
                tracing::warn!("failed to setup drm output: {err}");
                return;
            }
        };

        let disable_direct_scanout = std::env::var("TRAYLE_DISABLE_DIRECT_SCANOUT").is_ok();

        let dmabuf_feedback = drm_output.with_compositor(|compositor|{
            compositor.set_debug_flags(self.backend.debug_flags);

            get_surface_dmabuf_feedback(
                self.backend.primary_gpu,
                device.render_node,
                &mut self.backend.gpus,
                compositor.surface(),
            )
        });

        let surface = SurfaceData {
            dh: self.backend.dh.clone(),
            device_id: node,
            render_node: device.render_node,
            global: Some(global),
            drm_output,
            disable_direct_scanout,
            dmabuf_feedback,
        };

        device.surfaces.insert(crtc, surface);

        // kick-off rendering
        self.backend.loop_handle.insert_idle(move|state|{
            state.render_surface(node, crtc, state.clock.now());
        });
    }

    fn connector_disconnected(&mut self, node: DrmNode, connector: connector::Info, crtc: crtc::Handle) {
        let Some(device) = self.backend.backends.get_mut(&node) else {
            return;
        };

        if let Some(pos) = device
            .non_desktop_connectors
            .iter()
            .position(|(handle,_)|*handle==connector.handle())
        {
            let _ = device.non_desktop_connectors.remove(pos);
            if let Some(leasing_state) = device.leasing_global.as_mut() {
                leasing_state.withdraw_connector(connector.handle());
            }
        } else {
            device.surfaces.remove(&crtc);

            let output = self.space
                .outputs()
                .find(|o|{
                    o.user_data()
                        .get::<UdevOutputId>()
                        .map(|id|id.device_id == node && id.crtc == crtc)
                        .unwrap_or(false)
                })
                .cloned();

            if let Some(output) = output {
                self.space.unmap_output(&output);
            }
        }

        let mut renderer = self.backend.gpus.single_renderer(&device.render_node).unwrap();

        let _ = device.drm_output_manager.try_to_restore_modifiers::<_, OutputRenderElements<
            UdevRenderer<'_>,
            WindowRenderElement<UdevRenderer<'_>>,
        >>(
            &mut renderer,
            // FIXME: For a flicker free operation we should return the actual elements for this output..
            // Instead we just use black to "simulate" a modeset :)
            &DrmOutputRenderElements::default(),
        );
    }

    fn render(&mut self, node: DrmNode, crtc: Option<crtc::Handle>, frame_target: Time<Monotonic>) {
        let device_backend = match self.backend.backends.get_mut(&node) {
            Some(backend) => backend,
            None => {
                tracing::error!("trying to render on non-existent backend {node}");
                return;
            }
        };

        match crtc {
            Some(crtc) => {
                self.render_surface(node, crtc, frame_target);
            },
            None => {
                let crtcs = device_backend.surfaces.keys().copied().collect::<Vec<_>>();
                for crtc in crtcs {
                    self.render_surface(node, crtc, frame_target);
                }
            },
        }
    }

    fn render_surface(&mut self, node: DrmNode, crtc: crtc::Handle, frame_target: Time<Monotonic>) {
        let Some(output) = self
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

        self.pre_repaint(&output, frame_target);

        let Some(device) = self.backend.backends.get_mut(&node) else {
            return;
        };

        let Some(surface) = device.surfaces.get_mut(&crtc) else {
            return;
        };

        let start = Instant::now();

        // TODO: get scale from render surface when supporting HiDPI

        let frame = self
            .backend
            .pointer_image
            .get_image(1 /*scale*/, self.clock.now().into());

        let render_node = surface.render_node;
        let primary_gpu = self.backend.primary_gpu;
        let mut renderer = if primary_gpu == render_node {
            self.backend.gpus.single_renderer(&render_node)
        } else {
            let format = surface.drm_output.format();
            self.backend.gpus.renderer(&primary_gpu, &render_node, format)
        }.unwrap();

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

        let result = render_surface(
            surface,
            &mut renderer,
            &self.space,
            &output,
            self.pointer.current_location(),
            &pointer_image,
            &mut self.backend.pointer_element,
            self.dnd_icon.as_ref(),
            &mut self.cursor_status,
            self.show_window_preview
        );

        let reschedule = match result {
            Ok((has_rendered, states)) => {
                let dmabuf_feedback = surface.dmabuf_feedback.clone();
                self.post_repaint(&output, frame_target, dmabuf_feedback, &states);
                !has_rendered
            }
            Err(err) => {
                tracing::warn!("failed to render: {err:?}");
                match err {
                    SwapBuffersError::AlreadySwapped => false,
                    SwapBuffersError::TemporaryFailure(err) => match err.downcast_ref::<DrmError>() {
                        Some(DrmError::DeviceInactive) => true,
                        Some(DrmError::Access(DrmAccessError { source, .. })) => {
                            source.kind() == std::io::ErrorKind::PermissionDenied
                        }
                        _ => false,
                    },
                    SwapBuffersError::ContextLost(err) => match err.downcast_ref::<DrmError>() {
                        Some(DrmError::TestFailed(_)) => {
                            // reset the complete state, disabling all connectors and planes in case we hit a test failed
                            // most likely we hit this after a tty switch when a foreign master changed CRTC <-> connector bindings
                            // and we run in a mismatch
                            device
                                .drm_output_manager
                                .device_mut()
                                .reset_state()
                                .expect("failed to reset drm device");
                            true
                        }
                        _ => panic!("rendering loop lost: {err}"),
                    },
                }
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
                next_frame_target).saturating_sub(self.clock.now().into()
            );

            tracing::trace!("reschedule repaint timer with delay {reschedule_timeout:?} on {crtc:?}");

            let timer = Timer::from_duration(reschedule_timeout);
            self.backend.loop_handle.insert_source(timer, move|_,_,data|{
                data.render(node, Some(crtc), next_frame_target);
                TimeoutAction::Drop
            })
            .expect("failed to reschedule frame timer");
        } else {
            let elapsed = start.elapsed();
            tracing::trace!(?elapsed, "rendered surface");
        }
    }
}

// Backend Substate

struct BackendData {
    #[allow(dead_code)]
    token: RegistrationToken,
    drm_output_manager: DrmOutputManager<
        GbmAllocator<DrmDeviceFd>,
        GbmDevice<DrmDeviceFd>,
        Option<OutputPresentationFeedback>,
        DrmDeviceFd,
    >,
    drm_scanner: DrmScanner,
    non_desktop_connectors: Vec<(connector::Handle, crtc::Handle)>,
    render_node: DrmNode,
    surfaces: HashMap<crtc::Handle, SurfaceData>,
    leasing_global: Option<DrmLeaseState>,
    active_leases: Vec<DrmLease>,
}

#[allow(dead_code)]
struct SurfaceData {
    dh: DisplayHandle,
    device_id: DrmNode,
    render_node: DrmNode,
    global: Option<GlobalId>,
    drm_output: DrmOutput<
        GbmAllocator<DrmDeviceFd>,
        GbmDevice<DrmDeviceFd>,
        Option<OutputPresentationFeedback>,
        DrmDeviceFd,
    >,
    disable_direct_scanout: bool,
    dmabuf_feedback: Option<SurfaceDmabufFeedback>,
}

#[derive(Debug,PartialEq)]
struct UdevOutputId {
    device_id: DrmNode,
    crtc: crtc::Handle,
}

fn get_surface_dmabuf_feedback(
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

fn render_surface<'a>(
    surface: &'a mut SurfaceData,
    renderer: &mut UdevRenderer<'a>,
    space: &Space<WindowElement>,
    output: &Output,
    pointer_location: Point<f64, Logical>,
    pointer_image: &MemoryRenderBuffer,
    pointer_element: &mut PointerElement,
    dnd_icon: Option<&DndIcon>,
    cursor_status: &mut CursorImageStatus,
    show_window_preview: bool,
) -> Result<(bool, RenderElementStates), SwapBuffersError> {
    let output_geometry = space.output_geometry(output).unwrap();
    let scale = Scale::from(output.current_scale().fractional_scale());

    let mut custom_elements = Vec::<CustomRenderElements<_>>::new();

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

    let (elements, clear_color) = output_elements(
        output, space, custom_elements, renderer, show_window_preview
    );

    let frame_mode = match surface.disable_direct_scanout {
        true => FrameFlags::empty(),
        false => FrameFlags::DEFAULT,
    };

    let (rendered, states) = surface
        .drm_output
        .render_frame(renderer, &elements, clear_color, frame_mode)
        .map(|render_frame_result|{
            // renderer_sync feature
            (!render_frame_result.is_empty,render_frame_result.states)
        })
        .map_err(|err|match err {
            RenderFrameError::PrepareFrame(err) => SwapBuffersError::from(err),
            RenderFrameError::RenderFrame(OutputDamageTrackerError::Rendering(err)) => {
                SwapBuffersError::from(err)
            }
            _ => unreachable!()
        })?;

    crate::state::update_primary_scanout_output(
        space, output, dnd_icon, cursor_status, &states,
    );

    if rendered {
        let output_presentation_feedback = crate::state::take_presentation_feedback(output, space, &states);
        surface
            .drm_output
            .queue_frame(Some(output_presentation_feedback))
            .map_err(Into::<SwapBuffersError>::into)?;
    }

    Ok((rendered,states))
}



