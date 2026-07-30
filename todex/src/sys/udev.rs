/// `libudev` binding.
use std::ffi::{CStr, c_char, c_void};
use std::ptr::NonNull;

use crate::sys::error::{ErrCode, OsError, ResCode, simple_os_error};
use crate::sys::macros::simple_ffi;

// ===== Udev =====

/// Udev context.
#[repr(transparent)]
pub struct Udev(UdevPtr);

simple_ffi!(impl Drop for Udev::udev_unref);
simple_ffi!(impl Clone for Udev::udev_ref);
simple_ffi!(impl Debug for Udev);

impl Udev {
    /// Create new udev context.
    #[inline]
    pub fn new() -> Result<Self, UdevError> {
        unsafe { udev_new() }.ok_or_else(<_>::errno)
    }

    pub(crate) fn as_ptr(&self) -> NonNull<c_void> {
        self.0
    }

    /// Create new udev enumerator.
    #[inline]
    pub fn enumerate(&self) -> Result<Enumerate, EnumerateError> {
        Enumerate::new(self)
    }
}

// ===== Device =====

/// Udev context.
#[repr(transparent)]
pub struct Device(DevicePtr);

simple_ffi!(impl Drop for Device::udev_device_unref);
simple_ffi!(impl Clone for Device::udev_device_ref);
simple_ffi!(impl Debug for Device);

impl Device {
    /// Create new udev device from `syspath`.
    #[inline]
    pub fn from_syspath(udev: &Udev, syspath: &CStr) -> Result<Self, DeviceError> {
        unsafe { udev_device_new_from_syspath(udev.0, syspath.as_ptr()) }
            .ok_or_else(DeviceError::errno)
    }

    /// Returns the sysfs path of this device, including the `/sys` prefix.
    ///
    /// Example: `/sys/devices/virtual/tty/tty7`.
    #[inline]
    pub fn syspath(&self) -> &CStr {
        unsafe { CStr::from_ptr(udev_device_get_syspath(self.0)) }
    }

    /// Returns the sysfs name of this device, i.e the last component of the sysfs path.
    ///
    /// Example: `tty7` for the device `/sys/devices/virtual/tty/tty7`.
    #[inline]
    pub fn sysname(&self) -> &CStr {
        unsafe { CStr::from_ptr(udev_device_get_sysname(self.0)) }
    }

    /// Returns the sysfs device number of this device, i.e the numeric suffix.
    ///
    /// Example: `7` for the device `/sys/devices/virtual/tty/tty7`.
    #[inline]
    pub fn sysnum(&self) -> &CStr {
        unsafe { CStr::from_ptr(udev_device_get_sysnum(self.0)) }
    }

    /// Returns the kernel subsystem of this device.
    ///
    /// Example: `tty`, `block`, or `net`.
    #[inline]
    pub fn subsystem(&self) -> &CStr {
        unsafe { CStr::from_ptr(udev_device_get_subsystem(self.0)) }
    }

    /// Returns the sysfs path of this device, excluding the `/sys` prefix.
    ///
    /// Example: `/devices/virtual/tty/tty7`.
    #[inline]
    pub fn devpath(&self) -> &CStr {
        unsafe { CStr::from_ptr(udev_device_get_devpath(self.0)) }
    }

    /// Returns the device type of this device.
    ///
    /// Example: for devices of the `block` subsystem this can be `disk` or `partition`.
    #[inline]
    pub fn devtype(&self) -> &CStr {
        unsafe { CStr::from_ptr(udev_device_get_devtype(self.0)) }
    }

    /// Returns the device node path of this device.
    ///
    /// Example: for `/sys/devices/virtual/tty/tty7` the string `/dev/tty7` is typically returned.
    #[inline]
    pub fn devnode(&self) -> Option<&CStr> {
        unsafe {
            let ptr = udev_device_get_devnode(self.0);
            (!ptr.is_null()).then(|| CStr::from_ptr(ptr))
        }
    }
}

// ===== Enumerate =====

/// Udev enumerator.
#[repr(transparent)]
pub struct Enumerate(EnumeratePtr);

simple_ffi!(impl Drop for Enumerate::udev_enumerate_unref);
simple_ffi!(impl Clone for Enumerate::udev_enumerate_ref);
simple_ffi!(impl Debug for Enumerate);

// https://github.com/systemd/systemd/blob/main/src/libudev/libudev-enumerate.c
// based on the source, inside Enumerate, the udev ref count did not altered, even on drop
// although it seems that the field is unused internally

impl Enumerate {
    /// Create new udev enumerator.
    #[inline]
    pub fn new(udev: &Udev) -> Result<Self, EnumerateError> {
        unsafe { udev_enumerate_new(udev.0) }.ok_or_else(<_>::errno)
    }

    /// Add given `subsystem` as entry matches.
    #[inline]
    pub fn add_match_subsystem(&mut self, subsystem: &CStr) -> Result<(), MatchError> {
        unsafe { udev_enumerate_add_match_subsystem(self.0, subsystem.as_ptr()) }.result()
    }

    /// Add given `sysname` as entry matches.
    #[inline]
    pub fn add_match_sysname(&mut self, sysname: &CStr) -> Result<(), MatchError> {
        unsafe { udev_enumerate_add_match_sysname(self.0, sysname.as_ptr()) }.result()
    }

    /// Scan for devices with previously specified matches.
    #[inline]
    pub fn scan_devices(&mut self) -> Result<(), ScanError> {
        unsafe { udev_enumerate_scan_devices(self.0) }.result()
    }

    /// Get the list entry of scan result.
    #[inline]
    pub fn get_list_entry(&mut self) -> Result<Option<ListEntry>, ScanError> {
        match unsafe { udev_enumerate_get_list_entry(self.0) } {
            Some(list) => Ok(Some(ListEntry(list))),
            None => {
                // based on the source
                let errno = ErrCode::raw_errno();
                if errno == libc::ENODATA {
                    Ok(None)
                } else {
                    Err(ScanError(ErrCode::new(errno)))
                }
            }
        }
    }
}

// ===== ListEntry =====

/// Udev enumerate list entry.
#[repr(transparent)]
pub struct ListEntry(ListEntryPtr);

simple_ffi!(impl Debug for ListEntry);

impl ListEntry {
    /// Get the next entry, or `None` if no more entry available.
    #[inline]
    pub fn next(self) -> Option<ListEntry> {
        unsafe { udev_list_entry_get_next(self.0) }
    }

    /// Returns the entry name.
    #[inline]
    pub fn name(&self) -> &CStr {
        // internally, name is asserted to be non null
        unsafe { CStr::from_ptr(udev_list_entry_get_name(self.0)) }
    }

    /// Returns the entry value.
    #[inline]
    pub fn value(&self) -> Option<&CStr> {
        unsafe {
            let val = udev_list_entry_get_value(self.0);
            (!val.is_null()).then(|| CStr::from_ptr(val))
        }
    }
}

// ===== error =====

/// An error that can occur during udev context creation.
#[derive(Clone, Copy)]
pub struct UdevError(ErrCode);

simple_os_error!(UdevError, "create udev context");

/// An error that can occur during udev device creation.
#[derive(Clone, Copy)]
pub struct DeviceError(ErrCode);

simple_os_error!(DeviceError, "create udev device");

/// An error that can occur during udev enumerator creation.
#[derive(Clone, Copy)]
pub struct EnumerateError(ErrCode);

simple_os_error!(EnumerateError, "create udev enumerator");

/// An error that can occur during udev enumerator matching.
#[derive(Clone, Copy)]
pub struct MatchError(ErrCode);

simple_os_error!(MatchError, "add enumerator matches");

/// An error that can occur during udev enumerator scanning.
#[derive(Clone, Copy)]
pub struct ScanError(ErrCode);

simple_os_error!(ScanError, "scan enumerator");

// ===== ffi =====

type UdevPtr = NonNull<c_void>;

type DevicePtr = NonNull<c_void>;

type EnumeratePtr = NonNull<c_void>;

type ListEntryPtr = NonNull<c_void>;

unsafe extern "C" {
    fn udev_new() -> Option<Udev>;
    fn udev_ref(udev: UdevPtr) -> UdevPtr;
    fn udev_unref(udev: UdevPtr) -> UdevPtr;

    fn udev_device_new_from_syspath(udev: UdevPtr, syspath: *const c_char) -> Option<Device>;
    fn udev_device_ref(p: DevicePtr) -> DevicePtr;
    fn udev_device_unref(p: DevicePtr) -> DevicePtr;
    fn udev_device_get_syspath(udev_device: DevicePtr) -> *const c_char;
    fn udev_device_get_sysname(udev_device: DevicePtr) -> *const c_char;
    fn udev_device_get_sysnum(udev_device: DevicePtr) -> *const c_char;
    fn udev_device_get_subsystem(udev_device: DevicePtr) -> *const c_char;
    fn udev_device_get_devpath(udev_device: DevicePtr) -> *const c_char;
    fn udev_device_get_devtype(udev_device: DevicePtr) -> *const c_char;
    fn udev_device_get_devnode(udev_device: DevicePtr) -> *const c_char;

    fn udev_enumerate_new(udev: UdevPtr) -> Option<Enumerate>;
    fn udev_enumerate_ref(enums: EnumeratePtr) -> EnumeratePtr;
    fn udev_enumerate_unref(enums: EnumeratePtr) -> EnumeratePtr;
    fn udev_enumerate_add_match_subsystem(
        udev_enumerate: EnumeratePtr,
        subsystem: *const c_char,
    ) -> ResCode;
    fn udev_enumerate_add_match_sysname(
        udev_enumerate: EnumeratePtr,
        sysname: *const c_char,
    ) -> ResCode;
    fn udev_enumerate_scan_devices(udev_enumerate: EnumeratePtr) -> ResCode;
    fn udev_enumerate_get_list_entry(udev_enumerate: EnumeratePtr) -> Option<ListEntryPtr>;

    fn udev_list_entry_get_next(list: ListEntryPtr) -> Option<ListEntry>;
    // fn udev_list_entry_get_by_name(list: *mut udev_list_entry, name: *const c_char) -> *mut udev_list_entry;
    fn udev_list_entry_get_name(list: ListEntryPtr) -> *const c_char;
    fn udev_list_entry_get_value(list: ListEntryPtr) -> *const c_char;
}
