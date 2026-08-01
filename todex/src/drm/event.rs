// ===== Event =====

/// DRM events header.
///
/// This struct is a header for events written back to user-space on the DRM FD. A read on the DRM
/// FD will always only return complete events: e.g. if the read buffer is 100 bytes large and there
/// are two 64 byte events pending, only one will be returned.
///
/// @type: event type.
/// @length: total number of payload bytes (including header).
///
/// Event types 0 - 0x7fffffff are generic DRM events, 0x80000000 and up are chipset specific.
/// Generic DRM events include &DRM_EVENT_VBLANK, &DRM_EVENT_FLIP_COMPLETE and
/// &DRM_EVENT_CRTC_SEQUENCE.
#[derive(Debug)]
pub struct Event {
    /// Event type.
    pub ty: EventType,
    /// Total message length.
    pub len: u32,
}

impl Event {
    pub fn decode(bytes: &[u8]) -> Option<(Self, &[u8])> {
        let (header, rest) = bytes.split_first_chunk::<8>()?;
        let ty = u32::from_ne_bytes(*header[..4].first_chunk().unwrap());
        let ty = match ty {
            0x01 => EventType::Vblank,
            0x02 => EventType::FlipComplete,
            0x03 => EventType::CrtcSequence,
            ev => panic!("unknown drm event: {ev}"),
        };
        let len = u32::from_ne_bytes(*header[4..].first_chunk().unwrap());
        Some((Self { ty, len }, rest.get(..(len - 8) as usize)?))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EventType {
    /// Vertical blanking event
    ///
    /// This event is sent in response to `DRM_IOCTL_WAIT_VBLANK` with the `DRM_VBLANK_EVENT` flag
    /// set.
    ///
    /// The event payload is a [`Vblank`].
    Vblank = 0x01,
    /// Page-flip completion event
    ///
    /// This event is sent in response to an atomic commit or legacy page-flip with the
    /// `DRM_MODE_PAGE_FLIP_EVENT` flag set.
    ///
    /// The event payload is a struct [`Vblank`].
    FlipComplete = 0x02,
    /// CRTC sequence event.
    ///
    /// This event is sent in response to `DRM_IOCTL_CRTC_QUEUE_SEQUENCE`.
    ///
    /// The event payload is a struct [`CrtcSequence`].
    CrtcSequence = 0x03,
}

// ===== Vblank =====

#[derive(Debug, Clone)]
pub struct Vblank {
    // struct drm_event base;
    pub user_data: u64,
    pub tv_sec: u32,
    pub tv_usec: u32,
    pub sequence: u32,
    /// 0 on older kernels that do not support this
    pub crtc_id: u32,
}

impl Vblank {
    pub fn decode(mut slice: &[u8]) -> Option<Self> {
        Some(Self {
            user_data: decode!(&mut slice, u64),
            tv_sec: decode!(&mut slice, u32),
            tv_usec: decode!(&mut slice, u32),
            sequence: decode!(&mut slice, u32),
            crtc_id: decode!(slice, u32),
        })
    }
}

// ===== CrtcSequence =====

/// Event delivered at sequence.
///
/// Time stamp marks when the first pixel of the refresh cycle leaves the display engine for the
/// display
pub struct CrtcSequence {
    pub user_data: u64,
    pub time_ns: i64,
    pub sequence: u64,
}

impl CrtcSequence {
    pub fn decode(mut slice: &[u8]) -> Option<Self> {
        Some(Self {
            user_data: decode!(&mut slice, u64),
            time_ns: decode!(&mut slice, i64),
            sequence: decode!(slice, u64),
        })
    }
}

// ===== helpers =====

macro_rules! decode {
    (&mut $s:ident, $int:ty) => {{
        let (lead, trail) = $s.split_first_chunk()?;
        $s = trail;
        <$int>::from_ne_bytes(*lead)
    }};
    ($s:ident, $int:ty) => {
        <$int>::from_ne_bytes(*$s.as_array()?)
    };
}
use decode;
