use std::ffi::{CStr, c_char, c_double, c_int, c_uint, c_void};
use std::ptr::NonNull;

// ===== DevicePtr =====

#[repr(transparent)]
pub struct DevicePtr(NonNull<c_void>);

impl DevicePtr {
    fn copied(&self) -> Self {
        Self(self.0)
    }
}

impl DevicePtr {
    /// Returns the device name.
    ///
    /// The descriptive device name as advertised by the kernel and/or the hardware itself. To get
    /// the sysname for this device, use [`sysname()`].
    ///
    /// [`sysname()`]: Self::sysname
    #[inline]
    pub fn name(&self) -> &CStr {
        // The string may be the empty string but is never NULL.
        unsafe { CStr::from_ptr(libinput_device_get_name(self.copied())) }
    }

    /// Returns the system name of the device.
    ///
    /// To get the descriptive device name, use [`name()`].
    ///
    /// [`name()`]: Self::name
    #[inline]
    pub fn sysname(&self) -> &CStr {
        unsafe { CStr::from_ptr(libinput_device_get_sysname(self.copied())) }
    }

    /// Returns the bus type ID of this device.
    ///
    /// See `libinput/linux/input.h`.
    #[inline]
    pub fn id_bustype(&self) -> u32 {
        unsafe { libinput_device_get_id_bustype(self.copied()) }
    }

    /// Returns the product ID of this device.
    #[inline]
    pub fn id_product(&self) -> u32 {
        unsafe { libinput_device_get_id_product(self.copied()) }
    }

    /// Returns the vendor ID of this device.
    #[inline]
    pub fn id_vendor(&self) -> u32 {
        unsafe { libinput_device_get_id_vendor(self.copied()) }
    }

    /// Update the LEDs on the device, if any.
    ///
    /// If the device does not have LEDs, or does not have one or more of the LEDs given in the
    /// mask, this method does nothing.
    #[inline]
    pub fn led_update(&mut self, led: Led) {
        unsafe { libinput_device_led_update(self.copied(), led) };
    }

    /// Return `true` if this device has specified capability.
    #[inline]
    pub fn has_capability(&self, cap: Capability) -> bool {
        // return Non-zero if the given device has the capability or zero otherwise
        unsafe { libinput_device_has_capability(self.copied(), cap) != 0 }
    }

    /// Returns the physical size of a device in mm, where meaningful.
    ///
    /// This method only succeeds on devices with the required data, i.e. tablets, touchpads and
    /// touchscreens.
    #[inline]
    pub fn size(&self) -> Option<(f64, f64)> {
        let mut w = 0.;
        let mut h = 0.;
        let res = unsafe { libinput_device_get_size(self.copied(), &mut w, &mut h) };
        if res == 0 {
            Some((w, h))
        } else {
            None
        }
    }
}

// ===== DeviceRef =====

#[repr(transparent)]
pub struct DeviceRef<'a> {
    ptr: DevicePtr,
    _p: std::marker::PhantomData<&'a mut ()>
}

impl<'a> DeviceRef<'a> {
    pub(super) fn new(ptr: DevicePtr) -> Self {
        Self { ptr, _p: std::marker::PhantomData }
    }

    /// Converts into owned device.
    #[inline]
    pub fn into_owned(self) -> Device {
        Device::new(self.ptr)
    }
}

impl std::ops::Deref for DeviceRef<'_> {
    type Target = DevicePtr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.ptr
    }
}

impl std::ops::DerefMut for DeviceRef<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ptr
    }
}

// ===== Device =====

#[repr(transparent)]
pub struct Device(DevicePtr);

impl Drop for Device {
    #[inline]
    fn drop(&mut self) {
        unsafe { libinput_device_unref(self.0.copied()) };
    }
}

impl Clone for Device {
    #[inline]
    fn clone(&self) -> Self {
        Self::new(self.0.copied())
    }
}

impl Device {
    fn new(ptr: DevicePtr) -> Self {
        Self(unsafe { libinput_device_ref(ptr) })
    }
}

impl std::ops::Deref for Device {
    type Target = DevicePtr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Device {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// ===== enums =====

/// Mask reflecting LEDs on a device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Led(i32);

impl Led {
    pub const NUM_LOCK: Self = Self(1);
    pub const CAPS_LOCK: Self = Self(1 << 1);
    pub const SCROLL_LOCK: Self = Self(1 << 2);
    pub const COMPOSE: Self = Self(1 << 3);
    pub const KANA: Self = Self(1 << 4);
}

macro_rules! impl_ops {
    ($(impl $tr:ident::$fn:ident;)*) => {$(
        impl std::ops::$tr for Led {
            type Output = Self;

            #[inline]
            fn $fn(self, rhs: Self) -> Self::Output {
                Self(self.0.$fn(rhs.0))
            }
        }
    )*};
}
impl_ops! {
    impl BitOr::bitor;
    impl BitXor::bitxor;
    impl BitAnd::bitand;
}

/// Capabilities on a device.
///
/// A device may have one or more capabilities at a time, capabilities remain static for the
/// lifetime of the device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum Capability {
    Keyboard = 0,
    Pointer = 1,
    Touch = 2,
    TabletTool = 3,
    TabletPad = 4,
    Gesture = 5,
    Switch = 6,
}

impl Capability {
    /// An array of one for each capabilities.
    pub const ENTRIES: [Capability; 7] = [
        Self::Keyboard,
        Self::Pointer,
        Self::Touch,
        Self::TabletTool,
        Self::TabletPad,
        Self::Gesture,
        Self::Switch,
    ];
}

// ===== ffi =====

unsafe extern "C" {
    fn libinput_device_ref(dev: DevicePtr) -> DevicePtr;
    fn libinput_device_unref(dev: DevicePtr) -> DevicePtr;

    // fn libinput_device_set_user_data(dev: DevicePtr, data: *mut c_void);
    // fn libinput_device_get_user_data(dev: DevicePtr) -> *mut c_void;
    // fn libinput_device_get_device_group(dev: DevicePtr) -> GroupPtr;

    fn libinput_device_get_sysname(dev: DevicePtr) -> *const c_char;
    fn libinput_device_get_name(dev: DevicePtr) -> *const c_char;
    fn libinput_device_get_id_bustype(dev: DevicePtr) -> c_uint;
    fn libinput_device_get_id_product(dev: DevicePtr) -> c_uint;
    fn libinput_device_get_id_vendor(dev: DevicePtr) -> c_uint;

    fn libinput_device_led_update(dev: DevicePtr, leds: Led);
    fn libinput_device_has_capability(dev: DevicePtr, cap: Capability) -> c_int;
    fn libinput_device_get_size(dev: DevicePtr, w: &mut c_double, h: &mut c_double) -> c_int;
    // fn libinput_device_pointer_has_button(dev: DevicePtr, code: u32) -> c_int;
    // fn libinput_device_keyboard_has_key(dev: DevicePtr, code: u32) -> c_int;
    // fn libinput_device_touch_get_touch_count(dev: DevicePtr) -> c_int;
    // fn libinput_device_switch_has_switch(dev: DevicePtr, sw: Switch) -> c_int;

    // fn libinput_device_config_tap_get_default_enabled(dev: DevicePtr) -> TapState;
}
