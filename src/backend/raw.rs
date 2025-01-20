//! testing rendering to linux direct rendering manager

use std::time::Duration;

use anyhow::{Context, Result};
use smithay::{
    backend::{
        allocator::gbm::GbmDevice,
        drm::DrmDeviceFd,
        egl::{EGLContext, EGLDisplay},
        renderer::{gles::GlesRenderer, Color32F, Frame, Renderer},
        session::{libseat::LibSeatSession, Session},
        udev::UdevBackend,
    },
    reexports::rustix::fs::OFlags,
    utils::{DeviceFd, Rectangle, Transform},
};


pub fn run() -> Result<()> {

    let (mut session, _notifier) = LibSeatSession::new()?;
    let seat = session.seat();

    tracing::info!("session is active: {}",session.is_active());

    // let primary_gpu = udev::primary_gpu(&seat)?.context("no primary gpu")?;
    // let primary_gpu = DrmNode::from_path(primary_gpu)?;

    let udev = UdevBackend::new(&seat)?;

    let mut devices = udev.device_list();

    if let Some((device_id, path)) = devices.next() {
        tracing::debug!("using device {device_id} ({path:?})");

        let open_flags = OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOCTTY | OFlags::NONBLOCK;
        // let node = DrmNode::from_dev_id(device_id)?;

        let fd = session.open(&path, open_flags).context("failed to open device from session")?;
        let fd = DrmDeviceFd::new(DeviceFd::from(fd));
        // let (drm, _notifier) = DrmDevice::new(fd.clone(), true)?;
        let gbm = GbmDevice::new(fd)?;

        let egl_display = unsafe { EGLDisplay::new(gbm)? };
        // let egl_device = EGLDevice::device_for_display(&egl_display)?;
        let egl_context = EGLContext::new(&egl_display)?;

        let mut gles_renderer = unsafe { GlesRenderer::new(egl_context)? };

        let mut frame = Renderer::render(&mut gles_renderer, (1280,720).into(), Transform::Normal)?;

        // `trait Renderer`
        frame.draw_solid(
            Rectangle::from_size((640, 480).into()),
            &[],
            Color32F::new(1., 0., 0., 1.),
        )?;

        let _sync = frame.finish()?;

        std::thread::sleep(Duration::from_secs(3));
    } else {
        tracing::warn!("no available device detected");
    }

    for (device_id, path) in udev.device_list() {
        tracing::debug!("skipping device {device_id} ({path:?})");
    }

    tracing::info!("spit shet");

    Ok(())
}


