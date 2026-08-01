use crate::drm::ioctl::*;
use crate::drm::property::{Properties, Property, PropertyIter, WithProperties};
use crate::drm::plane::{Plane, PlaneType};
use crate::drm::{Crtc, Framebuffer, Handle};

impl WithProperties for Plane {
    type Properties = PlaneProperties;
}

/// Plane Properties.
///
/// Missing properties:
///
/// - "IN_FENCE_FD"
/// - "IN_FORMATS"
/// - "zpos"
///
/// Primary/Overlay sepecific:
/// - "COLOR_ENCODING"
/// - "COLOR_RANGE"
/// - "rotation"
pub struct PlaneProperties {
    // "type"
    pub ty: PlaneType,
    // "FD_ID"
    pub fb_id: Property<Option<Handle<Framebuffer>>>,
    // "CRTC_ID"
    pub crtc_id: Property<Option<Handle<Crtc>>>,
    // "CRTC_X"
    pub crtc_x: Property<u32>,
    // "CRTC_Y"
    pub crtc_y: Property<u32>,
    // "CRTC_W"
    pub crtc_w: Property<u32>,
    // "CRTC_H"
    pub crtc_h: Property<u32>,
    // "SRC_X"
    pub src_x: Property<u32>,
    // "SRC_Y"
    pub src_y: Property<u32>,
    // "SRC_W"
    pub src_w: Property<u32>,
    // "SRC_H"
    pub src_h: Property<u32>,
}

// yes this is caveman way of doing this, but this is temporary
const TY: u32 = 1;
const FB_ID: u32 = 1 << 1;
const CRTC_ID: u32 = 1 << 2;
const CRTC_X: u32 = 1 << 3;
const CRTC_Y: u32 = 1 << 4;
const CRTC_W: u32 = 1 << 5;
const CRTC_H: u32 = 1 << 6;
const SRC_X: u32 = 1 << 7;
const SRC_Y: u32 = 1 << 8;
const SRC_W: u32 = 1 << 9;
const SRC_H: u32 = 1 << 10;
const ALL: u32 =
    TY | FB_ID | CRTC_ID | CRTC_X | CRTC_Y | CRTC_W | CRTC_H | SRC_X | SRC_Y | SRC_W | SRC_H;

impl Properties<Plane> for PlaneProperties {
    fn from_raw_properties(mut props: PropertyIter<'_>) -> Result<Self, ErrCode> {
        let mut uninit = std::mem::MaybeUninit::<Self>::uninit();
        let mut init = 0u32;
        let ptr = uninit.as_mut_ptr();

        while let Some(prop) = props.next()? {
            macro_rules! id{($f:ident, $id:ident) => {
                unsafe {
                    (&raw mut (*ptr).$f).write(Property {
                        id: prop.id,
                        value: Handle::from_raw(prop.value as _),
                    });
                    $id
                }
            }}
            macro_rules! coord{($f:ident, $id:ident) => {
                unsafe {
                    (&raw mut (*ptr).$f).write(Property {
                        id: prop.id,
                        value: prop.value as _,
                    });
                    $id
                }
            }}

            init |= match prop.name.to_bytes() {
                b"type" => unsafe {
                    (&raw mut (*ptr).ty).write(match prop.value {
                        0 => PlaneType::Overlay,
                        1 => PlaneType::Primary,
                        2 => PlaneType::Cursor,
                        _ => continue,
                    });
                    TY
                },
                b"FB_ID" => id!(fb_id, FB_ID),
                b"CRTC_ID" => id!(crtc_id, CRTC_ID),
                b"CRTC_X" => coord!(crtc_x, CRTC_X),
                b"CRTC_Y" => coord!(crtc_y, CRTC_Y),
                b"CRTC_W" => coord!(crtc_w, CRTC_W),
                b"CRTC_H" => coord!(crtc_h, CRTC_H),
                b"SRC_X" => coord!(src_x, SRC_X),
                b"SRC_Y" => coord!(src_y, SRC_Y),
                b"SRC_W" => coord!(src_w, SRC_W),
                b"SRC_H" => coord!(src_h, SRC_H),
                _ => continue,
            };
        }

        if init != ALL {
            // return Err(Error::custom("missing plane properties"));
            todo!("errno")
        }

        Ok(unsafe { uninit.assume_init() })
    }
}


