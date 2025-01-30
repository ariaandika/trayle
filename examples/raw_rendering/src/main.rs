use std::os::{fd::AsFd, unix::prelude::*};

use drm::{control::{connector::State, Device as ControlDevice}, Device};
use gbm::{BufferObjectFlags, Format};

struct Card(libseat::Device);

impl AsFd for Card {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl Device for Card { }
impl ControlDevice for Card { }

fn main() {
    let _guard = setup_tracing();

    let mut session = libseat::Seat::open(|_,_|{}).unwrap();

    let udev = libudev::Context::new().unwrap();
    let mut scanner = libudev::Enumerator::new(&udev).unwrap();
    let device = scanner
        .scan_devices()
        .unwrap()
        .find(|device|matches!{
            (device.subsystem(),device.devnode()),
            (Some(sub),Some(_)) if sub == "drm"
        })
        .expect("no gpu");

    let drm = Card(session.open_device(&device.devnode().unwrap()).unwrap());

    let resource = drm.resource_handles().unwrap();
    let conn = resource.connectors().iter()
        .filter_map(|e|Some((*e,drm.get_connector(*e, true).ok()?)))
        .find(|(_,e)|e.state() == State::Connected)
        .expect("no connector");

    let crtc = resource.crtcs().iter()
        .filter_map(|e|Some((*e,drm.get_crtc(*e).ok()?)))
        .find(|(_,e)|e.mode().is_some())
        .expect("no crtc");

    let (w,h) = crtc.1.mode().unwrap().size();

    let gbm = gbm::Device::new(&drm).unwrap();
    let mut bo = gbm
        .create_buffer_object::<()>(w as u32, h as u32, Format::Argb8888, BufferObjectFlags::all())
        .unwrap();

    let fb = drm.add_framebuffer(&bo, 32, 32).unwrap();
    drm.set_crtc(crtc.0, Some(fb), (0,0), &[conn.0], crtc.1.mode()).unwrap();

    let size = (w as usize) * (h as usize) * 4;
    tracing::info!("Width: {w}, Height: {h}, Size: {size}");

    let mut buffer = vec![0u8;size];
    for (i,byte) in buffer.iter_mut().enumerate() {
        if i % 4 == 2 {
            *byte = i as u8;
        }
    }

    bo.write(&buffer).unwrap();
    wait(1300);

    // drop session to let libseat change buffer before current framebuffer destroyed
    // prevent blank screen blinking
    drop(session);
    drm.destroy_framebuffer(fb).unwrap();
}

fn wait(millis: u64) {
    std::thread::sleep(std::time::Duration::from_millis(millis));
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

