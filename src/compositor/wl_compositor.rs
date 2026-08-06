use std::fmt;
use todex::wayland::primitives::Version;
use wl_surface::*;

use crate::compositor::prelude::*;
use crate::compositor::error::{impl_from, WlError};
use crate::compositor::traits::CommitError;
use crate::surface::{Role, Surface, Surfaces};

// ===== wl_surface =====

const V5: Version = Version::new(5).unwrap();

pub fn attach(
    surface: &mut Surface,
    msg: Msg<Attach>,
    client: &mut ClientMut,
) -> Result<(), AttachError> {
    let surface = surface.pending_mut();
    let version = msg.version();
    let Attach { buffer, x, y } = msg.into_payload();

    surface.buffer = match buffer {
        Some(obj) => Some(client.get_with(obj)?.handle()),
        None => None,
    };

    if !matches!((x, y), (0, 0)) {
        // version is 5 or higher, passing any non-zero x or y is a protocol violation, and will
        // result in an 'invalid_offset' error being raised.
        if version >= V5 {
            return Err(wl_surface::Error::InvalidOffset.into());
        }
        surface.offset.0 += x;
        surface.offset.1 += y;
    }

    Ok(())
}

pub fn commit(
    msg: Msg<Commit>,
    client: &mut ClientMut,
    res: &mut Resources
) -> Result<(), CommitError> {
    let surface = &mut res.surfaces[msg.handle()];
    surface.swap_state();

    let is_configured = surface.is_configured();
    if is_configured {
        if let Some(handle) = surface.current_mut().buffer.take() {
            // TODO: temporary implementation, write surface as ppm file
            let buffer = &mut res.buffers[handle];
            let shm_pool = match buffer.factory {
                crate::shm::BufferFactory::ShmPool(handle) => &mut res.shm_pools[handle],
            };
            let pixels = shm_pool.as_slice();
            let mut file = std::fs::File::create("/tmp/img.ppm").unwrap();
            std::io::Write::write_all(&mut file, b"P6\n1280 720\n255\n").unwrap();
            for y in 0..buffer.height {
                let row = &pixels[(y * buffer.stride) as usize..];

                for x in 0..buffer.width {
                    let pixel = &row[(x * 4) as usize..(x * 4 + 4) as usize];

                    let b = pixel[0];
                    let g = pixel[1];
                    let r = pixel[2];

                    std::io::Write::write_all(&mut file, &[r, g, b]).unwrap();
                }
            }

            let wl_buffer = buffer.wl_buffer;
            client.send(wl_buffer.release());
        }

        // TODO: add separate state
        static START: std::sync::LazyLock<std::time::Instant> = std::sync::LazyLock::new(std::time::Instant::now);

        let timestamp = START.elapsed().as_millis() as u32;
        if let Some(callback) = surface.current_mut().request_frames.take() {
            client.send(callback.done(timestamp));
            client.delete_id(callback);
            client.objects.remove(callback)?;
        }
    } else {
        surface.set_configured();
    }

    let Some(role) = surface.role() else {
        return Ok(());
    };

    use crate::compositor::xdg_shell;

    match role {
        Role::XdgToplevel(obj) => {
            xdg_shell::toplevel_commit(is_configured, obj, client, &mut res.xdg_surfaces)
        }
    }
}

// TODO: wl_surface: handle stacking frame requests
pub fn frame(surface: &mut Surface, msg: Msg<Frame>, client: &mut ClientMut) {
    surface.pending_mut().request_frames = Some(client.create(msg));
}

// TODO: wl_surface: handle stacking release requests
pub fn get_release(
    surface: &mut Surface,
    msg: Msg<GetRelease>,
    client: &mut ClientMut,
) -> Result<(), wl_surface::Error> {
    let surface = surface.pending_mut();
    if surface.buffer.is_none() {
        return Err(wl_surface::Error::NoBuffer);
    }
    surface.request_release = Some(client.objects.create(msg));
    Ok(())
}

pub fn offset(msg: Msg<Offset>, surfaces: &mut Surfaces) {
    let (cr_x, cr_y) = surfaces[msg.handle()].current().offset;
    let pending = surfaces[msg.handle()].pending_mut();

    // The x and y arguments specify the location of the new pending buffer's upper left corner,
    // relative to the current buffer's upper left corner
    pending.offset.0 = cr_x + msg.x;
    pending.offset.1 = cr_y + msg.y;
}

// ===== Error =====

#[derive(Debug, Clone, Copy)]
pub enum AttachError {
    UnknownBuffer(UnknownId),
    Surface(wl_surface::Error),
}

impl WlError for AttachError {
    fn code(&self) -> u32 {
        match self {
            Self::UnknownBuffer(err) => err.code(),
            Self::Surface(err) => err.code(),
        }
    }

    fn message(&self) -> &str {
        match self {
            Self::UnknownBuffer(err) => err.message(),
            Self::Surface(err) => err.message(),
        }
    }
}

impl fmt::Display for AttachError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("cannot attach buffer: ")?;
        match self {
            Self::UnknownBuffer(err) => err.fmt(f),
            Self::Surface(err) => err.fmt(f),
        }
    }
}

impl_from! {
    impl AttachError;
    UnknownBuffer, UnknownId;
    Surface, wl_surface::Error;
}
