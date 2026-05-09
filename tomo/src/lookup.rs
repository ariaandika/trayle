#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum Interfaces {
    WlDisplay,
    WlRegistry,
    WlCallback,
    WlCompositor,
    WlShmPool,
    WlShm,
    WlBuffer,
    WlDataOffer,
    WlDataSource,
    WlDataDevice,
    WlDataDeviceManager,
    WlShell,
    WlShellSurface,
    WlSurface,
    WlSeat,
    WlPointer,
    WlKeyboard,
    WlTouch,
    WlOutput,
    WlRegion,
    WlSubcompositor,
    WlSubsurface,
    WlFixes,
}

impl Interfaces {
    #[inline]
    pub fn from_u32(int: u32) -> Option<Self> {
        if int < 23 {
            Some(unsafe { Self::from_u32_unchecked(int) })
                    } else {
            None
        }
    }

    /// # Safety
    ///
    /// `int` must be below `23`.
    #[inline]
    pub unsafe fn from_u32_unchecked(int: u32) -> Self {
        debug_assert!(int < 23);
        unsafe { std::mem::transmute(int) }
    }
}
