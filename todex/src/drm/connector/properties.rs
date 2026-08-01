use crate::sys::error::ErrCode;
use crate::drm::{Connector, Crtc, Handle};
use crate::drm::property::{Properties, Property, PropertyIter, WithProperties};

impl WithProperties for Connector {
    type Properties = ConnProperties;
}

/// Connector Properties.
///
/// Missing properties:
///
/// - "EDID"
/// - "DPMS"
/// - "link-status"
/// - "non-desktop"
/// - "TILE"
/// - "scaling mode"
/// - "underscan"
/// - "underscan hborder"
/// - "underscan vborder"
/// - "max bpc"
/// - "Colorspace"
/// - "HDR_OUTPUT_METADATA"
/// - "vrr_capable"
/// - "Content Protection"
/// - "HDCP Content Type"
/// - "Broadcast RGB" ?
/// - "content type" ?
/// - "panel_type" ?
/// - "adaptive backlight modulation" ?
pub struct ConnProperties {
    // "CRTC_ID"
    pub crtc_id: Property<Option<Handle<Crtc>>>,
}

impl Properties<Connector> for ConnProperties {
    #[inline]
    fn from_raw_properties(mut props: PropertyIter<'_>) -> Result<Self, ErrCode> {
        loop {
            match props.next()? {
                Some(prop) => {
                    if *prop.name == c"CRTC_ID" {
                        break Ok(Self {
                            crtc_id: Property {
                                id: prop.id,
                                value: Handle::from_raw(prop.value as _),
                            },
                        });
                    }
                }
                None => todo!("errno")// break Err(Error::custom("missing connector properties")),
            }
        }
    }
}
