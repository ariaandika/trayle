use std::{os::fd::{AsFd, OwnedFd}, path::Path};
use smithay::{
    backend::{
        allocator::{
            dmabuf::DmabufAllocator,
            gbm::{GbmAllocator, GbmBufferFlags, GbmDevice},
            Allocator, Fourcc, Modifier,
        },
        egl::{EGLContext, EGLDisplay},
        renderer::{
            gles::{GlesRenderbuffer, GlesRenderer},
            Bind, Color32F, Frame, ImportDma, Offscreen, Renderer,
        },
        session::{libseat::LibSeatSession, Session},
    },
    reexports::{
        drm::{control::{connector::State, Device as ControlDevice}, Device},
        rustix::fs::OFlags,
    },
    utils::{Physical, Rectangle, Size, Transform},
};

const GPU: &'static str = "/dev/dri/card1";
const GPU_OPEN_FLAGS: OFlags = OFlags::all();
const GBM_BUFFER_FLAGS: GbmBufferFlags = GbmBufferFlags::RENDERING;
const BUFFER_FORMAT: Fourcc = Fourcc::Argb8888;
const BUFFER_MODIFIER: [Modifier;1] = [Modifier::Linear];
const RENDER_TRANSFORM: Transform = Transform::Normal;

fn main() {
    // 1. Create Session
    let (mut session,_guard) = LibSeatSession::new().unwrap();
    let _guard = setup_tracing();

    // 2. Open device fd and setup Drm stuff
    let fd = Session::open(&mut session, Path::new(GPU), GPU_OPEN_FLAGS).unwrap();

    // smithay implementation, this setup the crtc stuff, maybe, requires a surface to enabled
    // from smithay [`drm::DrmDevice`]
    // > smithay enables connectors, when attached to a surface, and disables them, when detached.
    // ```
    // let drm_fd = DrmDeviceFd::new(fd.try_clone().unwrap().into());
    // let (_drm_device,_guard) = drm::DrmDevice::new(drm_fd, true).unwrap();
    // ```

    let device = DrmDevice(fd.try_clone().unwrap());
    let resource = device.resource_handles().unwrap();
    let conn = resource.connectors().iter()
        .filter_map(|e|Some((*e,device.get_connector(*e, true).ok()?)))
        .find(|(_,e)|e.state() == State::Connected)
        .unwrap();
    let crtc = resource.crtcs().iter()
        .filter_map(|e|Some((*e,device.get_crtc(*e).ok()?)))
        .find(|(_,e)|e.mode().is_some())
        .unwrap();
    device.set_crtc(crtc.0, None, (0,0), &[conn.0], crtc.1.mode()).unwrap();

    // 3. Create Allocator
    let gbm_device = GbmDevice::new(fd.try_clone().unwrap()).unwrap();
    let allocator = GbmAllocator::new(gbm_device, GBM_BUFFER_FLAGS);
    let mut allocator = DmabufAllocator(allocator);

    // 4. Create Renderer
    let native = GbmDevice::new(fd).unwrap();
    let display = unsafe { EGLDisplay::new(native) }.unwrap();
    let context = EGLContext::new(&display).unwrap();
    let mut renderer = unsafe { GlesRenderer::new(context) }.unwrap();

    // 5. Create Buffer Object from Allocator
    // following steps is generic, so no implementation specific allowed
    let size = Size::<i32, Physical>::from((640,480));
    let buffer = Allocator::create_buffer(
        &mut allocator,
        size.w as u32,
        size.h as u32,
        BUFFER_FORMAT,
        &BUFFER_MODIFIER,
    ).unwrap();

    // 6. Bind Renderer to Buffer Object
    Bind::bind(&mut renderer, buffer.clone()).unwrap();

    // 7. Kick off rendering
    let mut frame = Renderer::render(&mut renderer, size, RENDER_TRANSFORM).unwrap();
    let clear_color = Color32F::new(1., 0., 0., 1.);
    let clear_pos = [Rectangle::from_size(size)];

    Frame::clear(&mut frame, clear_color, &clear_pos).unwrap();
    Frame::finish(frame).unwrap().wait().unwrap();

    // 8. Additionally, dmabuf and offscreen rendering, idk what it is
    // probably should setup another renderer
    let texture = ImportDma::import_dmabuf(&mut renderer, &buffer, None).unwrap();
    let offscreen_buffer = Offscreen::<GlesRenderbuffer>::create_buffer(
        &mut renderer,
        BUFFER_FORMAT,
        Size::from((size.w,size.h)),
    ).unwrap();

    Bind::bind(&mut renderer, offscreen_buffer).unwrap();

    let mut frame = Renderer::render(&mut renderer, size, RENDER_TRANSFORM).unwrap();
    Frame::render_texture_at(
        &mut frame,
        &texture,
        (0,0).into(),
        1,
        1.,
        RENDER_TRANSFORM,
        &[Rectangle::from_size(size)],
        &[],
        1.
    ).unwrap();
    Frame::finish(frame).unwrap().wait().unwrap();

    wait(3);
}

struct DrmDevice(OwnedFd);

impl AsFd for DrmDevice {
    fn as_fd(&self) -> std::os::unix::prelude::BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl Device for DrmDevice { }
impl ControlDevice for DrmDevice { }



fn wait(secs: u64) {
    std::thread::sleep(std::time::Duration::from_secs(secs));
}

fn setup_tracing() -> tracing_appender::non_blocking::WorkerGuard {
    use tracing_appender::{rolling::never, non_blocking};
    std::fs::remove_file(".log").ok();
    let (log, guard) = non_blocking(never(".", ".log"));
    tracing_subscriber::fmt()
        .with_writer(log)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    guard
}
