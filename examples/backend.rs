use std::os::unix::prelude::OwnedFd;
use std::path::Path;

use smithay::backend::session::{Session, libseat::LibSeatSession};

use smithay::backend::allocator::gbm::{
    GbmAllocator, GbmBufferFlags, GbmDevice
};

use smithay::backend::allocator::Allocator;
use smithay::backend::renderer::{Frame, Renderer};

const FORMAT: smithay::reexports::gbm::Format = smithay::reexports::gbm::Format::Argb8888;
const TRANSFORM: smithay::utils::Transform = smithay::utils::Transform::Flipped180;

fn main() {
    let _guard = setup_tracing();

    let (mut session,_guard) = LibSeatSession::new().unwrap();

    let fd = setup_fd(&mut session);

    let mut allocator = setup_allocator(fd.try_clone().unwrap());
    let mut renderer = setup_renderer(fd);

    // ERROR: Os Error: Invalid Arguments
    let _buffer = Allocator::create_buffer(&mut allocator, 640, 480, FORMAT, &[]).unwrap();

    let mut frame = Renderer::render(&mut renderer, (640,480).into(), TRANSFORM).unwrap();

    std::thread::sleep(std::time::Duration::from_secs(2));

    let _ok = Frame::draw_solid(
        &mut frame,
        smithay::utils::Rectangle::new((0,0).into(), (540,480).into()),
        &[],
        smithay::backend::renderer::Color32F::new(1.0, 0.0, 0.0, 1.0),
    ).unwrap();

    std::thread::sleep(std::time::Duration::from_secs(2));

    let _sync = Frame::finish(frame).unwrap();

    std::thread::sleep(std::time::Duration::from_secs(2));

    drop(_guard);

    std::thread::sleep(std::time::Duration::from_secs(2));
}


fn setup_fd(session: &mut LibSeatSession) -> OwnedFd {
    use smithay::reexports::rustix::fs::OFlags;
    let fd = Session::open(session, Path::new("/dev/dri/card1"), OFlags::all()).unwrap();
    fd
}

type GbmAllocatorAlias = smithay::backend::allocator::gbm::GbmAllocator<OwnedFd>;

fn setup_allocator(fd: OwnedFd) -> GbmAllocatorAlias {
    let device = GbmDevice::new(fd).unwrap();
    let allocator = GbmAllocator::new(device, GbmBufferFlags::all());
    allocator
}

fn setup_renderer(fd: OwnedFd) -> smithay::backend::renderer::gles::GlesRenderer {
    use smithay::backend::egl::{
        context::EGLContext, EGLDisplay
    };
    use smithay::backend::renderer::gles:: GlesRenderer;


    let native = GbmDevice::new(fd).unwrap();
    let display = unsafe { EGLDisplay::new(native) }.unwrap();
    let context = EGLContext::new(&display).unwrap();
    let renderer = unsafe { GlesRenderer::new(context) }.unwrap();

    renderer
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

