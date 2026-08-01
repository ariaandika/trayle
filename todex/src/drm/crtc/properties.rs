use crate::sys::error::ErrCode;
use crate::drm::{Crtc, Handle};
use crate::drm::property::{Properties, Property, PropertyIter, WithProperties};
use crate::drm::connector::ModeInfo;

impl WithProperties for Crtc {
    type Properties = CrtcProperties;
}

/// Connector Properties.
///
/// Missing properties:
///
/// - "OUT_FENCE_PTR"
/// - "VRR_ENABLED"
/// - "DEGAMMA_LUT"
/// - "DEGAMMA_LUT_SIZE"
/// - "CTM"
/// - "GAMMA_LUT"
/// - "GAMMA_LUT_SIZE"
pub struct CrtcProperties {
    // "ACTIVE"
    pub active: Property<bool>,
    // "MODE_ID"
    /// can either be `Handle<ModeInfo>` or `Handle<Blob>`
    ///
    /// because `ObjectType` have a value for `MODE`, but when client whan to send as input, it
    /// needs to be in blob form
    pub mode_id: Property<Option<Handle<ModeInfo>>>,
}

const ACTIVE: u32 = 1;
const MODE_ID: u32 = 1 << 1;
const ALL: u32 = ACTIVE | MODE_ID;

impl Properties<Crtc> for CrtcProperties {
    #[inline]
    fn from_raw_properties(mut props: PropertyIter<'_>) -> Result<Self, ErrCode> {
        let mut uninit = std::mem::MaybeUninit::<Self>::uninit();
        let mut init = 0u32;
        let ptr = uninit.as_mut_ptr();

        while let Some(prop) = props.next()? {
            init |= match prop.name.to_bytes() {
                b"ACTIVE" => unsafe {
                    (&raw mut (*ptr).active).write(Property {
                        id: prop.id,
                        value: prop.value != 0,
                    });
                    ACTIVE
                },
                b"MODE_ID" => unsafe {
                    (&raw mut (*ptr).mode_id).write(Property {
                        id: prop.id,
                        value: Handle::from_raw(prop.value as _),
                    });
                    MODE_ID
                },
                _ => continue,
            };
        }

        if init != ALL {
            todo!("errno")
            // return Err(Error::custom("missing crtc properties"));
        }

        Ok(unsafe { uninit.assume_init() })
    }
}
