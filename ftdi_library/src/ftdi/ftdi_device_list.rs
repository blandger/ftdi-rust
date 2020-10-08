#![allow(non_camel_case_types)]

use libusb_sys as ffi;
use libc::{c_int,c_uchar};
use std::{mem::{MaybeUninit}, slice, ptr};
use log::{debug, info, error};
use snafu::{GenerateBacktrace};
use crate::ftdi::core::{FtdiError, Result};
use crate::ftdi::ftdi_context::ftdi_context;

/// brief list of usb devices created by ftdi_usb_find_all()
pub struct ftdi_device_list {
    /// Vector keeps all devices and all are freed later
    /// pub ftdi_device_list: Vec<*mut ffi::libusb_device>,
    /// found ans stored number of devices.
    /// It equals to number of devices in vector
    pub number_found_devices: usize,
    /// pointer to libusb's usb_device
    pub system_device_list: Option<*const *mut ffi::libusb_device>,
}
impl ftdi_device_list {
    /// Creates usb device list for all available devices in system
    pub fn new(ftdi: &ftdi_context) -> Result<Self> {
        debug!("start new ftdi_device_list...");
        // check ftdi context
        if ftdi.usb_ctx == None {
            let error = FtdiError::UsbInit {code: -100, message: "ftdi context is not initialized previously".to_string(),
                backtrace: GenerateBacktrace::generate()
            };
            error!("{}", error);
            return Err(error);
        }
        // fetch usb device list
        // let (device_list, devices_len) = ftdi_device_list::get_usb_device_list_internal(ftdi)?;
        let mut device_list_uninit: MaybeUninit::<*const *mut ffi::libusb_device> = MaybeUninit::uninit();

        let get_device_list_result = unsafe { ffi::libusb_get_device_list(ftdi.usb_ctx.unwrap(), device_list_uninit.as_mut_ptr()) };
        if get_device_list_result < 0 {
            let result = FtdiError::UsbCommandError { code: -5, message: "libusb_get_device_list() failed".to_string(),
                backtrace: GenerateBacktrace::generate()
            };
            error!("{}", result);
            return Err(result);
        }
        let device_list: *const *mut ffi::libusb_device = unsafe { device_list_uninit.assume_init() };
        debug!("found total usb device(s) quantity = [{}]", get_device_list_result);

        // common fields are filled
        let list = ftdi_device_list{
            number_found_devices: get_device_list_result as usize,
            system_device_list: Some(device_list)};
        debug!("found usb device quantity = {}", get_device_list_result);
        Ok(list)
    }

    /// Finds all ftdi devices with given VID:PID on the usb bus. Creates a new
    ///  ftdi_device_list which is deallocated automatically after use and going out of scope.
    ///  With VID:PID 0:0, it searches for the default devices
    ///  (0x403:0x6001, 0x403:0x6010, 0x403:0x6011, 0x403:0x6014, 0x403:0x6015)
    ///
    ///   param ftdi is ftdi_context to create
    ///   devlist is stored in devices field 'system_device_list' field
    ///   \param vendor Vendor ID to search for
    ///   \param product Product ID to search for
    /// ```rust, no_run
    /// use ::ftdi_library::ftdi::ftdi_context::ftdi_context;
    /// use ::ftdi_library::ftdi::ftdi_device_list::ftdi_device_list;
    /// use libc::{c_int};
    ///
    ///    let mut ftdi = ftdi_context::new(Some(4)).unwrap(); // ffi::LIBUSB_LOG_LEVEL_DEBUG
    ///    let mut ftdi_list = ftdi_device_list::new(&ftdi).unwrap();
    ///     match ftdi_list.ftdi_usb_find_all(&mut ftdi, 0, 0) {
    ///         Ok(ftdi_usb_list) => {
    ///             println!("ftdi_usb_list is OK, found FTDI system_device_list = {:?}", ftdi_usb_list.system_device_list);
    ///             println!("ftdi_list is OK, found FTDI number = {}", ftdi_usb_list.number_found_devices);
    ///         },
    ///         Err(internal_error) => {
    ///             println!("{:?}", internal_error);
    ///         },
    ///     }
    /// ```
    pub fn ftdi_usb_find_all(&mut self, ftdi: &mut ftdi_context, vendor: u16, product: u16) -> Result<Self> {
        debug!("start new ftdi_device_list by vendor = {}, product={} ...", vendor, product);
        // check ftdi context
        if ftdi.usb_ctx == None {
            let error = FtdiError::UsbInit {code: -100, message: "ftdi context is not initialized previously".to_string(),
                backtrace: GenerateBacktrace::generate()
            };
            error!("{}", error);
            return Err(error);
        }
        if self.system_device_list.is_none() && self.number_found_devices == 0 {
            let error = FtdiError::UsbInit {code: -101, message: "fftdi_device_list is not created previously".to_string(),
                backtrace: GenerateBacktrace::generate()
            };
            error!("{}", error);
            return Err(error);
        }
        // make slice using device list and iterate over it
        let sys_device_list = unsafe { slice::from_raw_parts(
            self.system_device_list.unwrap(), self.number_found_devices) };
        let mut usb_dev_index = 0;
        let mut found_usb_count = 0;
        // loop over devices
        for dev in sys_device_list {

            let speed = unsafe { ffi::libusb_get_device_speed(*dev) };
            let mut descriptor_uninit: MaybeUninit::<ffi::libusb_device_descriptor> = MaybeUninit::uninit();
            // get description from usb device
            let has_descriptor = match unsafe { ffi::libusb_get_device_descriptor(*dev, descriptor_uninit.as_mut_ptr()) } {
                0 => {
                    true
                },
                _err => {
                    error!("{}", FtdiError::UsbCommandError{code: -6, message: "libusb_get_device_descriptor() failed".to_string(),
                        backtrace: GenerateBacktrace::generate()
                    });
                    false
                },
            };
            let handle: *mut ffi::libusb_device_handle = ptr::null_mut();
            if has_descriptor {
                let descriptor: ffi::libusb_device_descriptor = unsafe { descriptor_uninit.assume_init() };
                info!("USB ID [{:?}] : {:04x}:{:04x}", usb_dev_index, descriptor.idVendor, descriptor.idProduct);
                // extract usb devices only specified by vendor and product ids
                if (vendor > 0 || product > 0 &&
                    descriptor.idVendor == vendor && descriptor.idProduct == product) ||
                    !(vendor > 0 || product > 0) &&
                        (descriptor.idVendor == 0x403) && (descriptor.idProduct == 0x6001 || descriptor.idProduct == 0x6010
                        || descriptor.idProduct == 0x6011 || descriptor.idProduct == 0x6014
                        || descriptor.idProduct == 0x6015) {
                    debug!("Process matched device [{}]", usb_dev_index);
                    print_debug_device_descriptor(handle, &descriptor, speed);
                    unsafe { ffi::libusb_ref_device(*dev) };
                    ftdi.usb_dev = Some(handle);
                    found_usb_count += 1; // count found
                } else {
                    debug!("SKIPPED unmatched USB ID [{:?}] : {:04x}:{:04x}", usb_dev_index, descriptor.idVendor, descriptor.idProduct);
                }
            }
            usb_dev_index += 1;
        }
        let list = ftdi_device_list{
            number_found_devices: found_usb_count,
            // system_device_list: Some(sys_device_list.as_ptr())
            system_device_list: None
        };
        if self.system_device_list != None {
            unsafe { ffi::libusb_free_device_list(self.system_device_list.unwrap(),1); };
            self.system_device_list = None;
        }
        debug!("usb device quantity: ftdi found = [{}], total usb found = [{}]", found_usb_count, usb_dev_index);
        Ok(list)
    }

    // Helper internal method to fetch usb device list.
    // Return tuple with found list and found device quantity
/*    fn get_usb_device_list_internal(ftdi: &ftdi_context) -> Result< (*const *mut ffi::libusb_device, isize) > {
        let mut device_list_uninit: MaybeUninit::<*const *mut ffi::libusb_device> = MaybeUninit::uninit();

        let get_device_list_result = unsafe { ffi::libusb_get_device_list(ftdi.usb_ctx.unwrap(), device_list_uninit.as_mut_ptr()) };
        if get_device_list_result < 0 {
            let result = FtdiError::UsbCommandError { code: -5, message: "libusb_get_device_list() failed".to_string(),
                backtrace: GenerateBacktrace::generate()
            };
            error!("{}", result);
            return Err(result);
        }
        let device_list: *const *mut ffi::libusb_device = unsafe { device_list_uninit.assume_init() };
        debug!("found total usb device(s) quantity = [{}]", get_device_list_result);
        Ok( (device_list, get_device_list_result) )
    }
*/
}
impl Drop for ftdi_device_list {
    fn drop(&mut self) {
        if self.system_device_list.is_some() {
            debug!("cleaning up ftdi_device_list...");
            unsafe { ffi::libusb_free_device_list(self.system_device_list.unwrap(), self.number_found_devices as c_int) };
            self.system_device_list = None;
            self.number_found_devices = 0;
        }
        debug!("cleaned up ftdi_device_list - OK");
    }
}


pub fn print_debug_device_descriptor(handle: *mut ffi::libusb_device_handle,
                                     descriptor: &ffi::libusb_device_descriptor,
                                     speed: c_int) {
    debug!("======= Device Descriptor: =======");
    debug!("  bLength: {:16}", descriptor.bLength);
    debug!("  bDescriptorType: {:8} {}", descriptor.bDescriptorType, get_descriptor_type(descriptor.bDescriptorType));
    debug!("  bcdUSB:            {:#06x} {}", descriptor.bcdUSB, get_bcd_version(descriptor.bcdUSB));
    debug!("  bDeviceClass:        {:#04x} {}", descriptor.bDeviceClass, get_class_type(descriptor.bDeviceClass));
    debug!("  bDeviceSubClass: {:8}", descriptor.bDeviceSubClass);
    debug!("  bDeviceProtocol: {:8}", descriptor.bDeviceProtocol);
    debug!("  bMaxPacketSize0: {:8}", descriptor.bMaxPacketSize0);
    debug!("  idVendor:          {:#06x}", descriptor.idVendor);
    debug!("  idProduct:         {:#06x}", descriptor.idProduct);
    debug!("  bcdDevice:         {:#06x}", descriptor.bcdDevice);
    debug!("  iManufacturer: {:10} {}", descriptor.iManufacturer, get_string_descriptor(handle, descriptor.iManufacturer).unwrap_or_default());
    debug!("  iProduct: {:15} {}", descriptor.iProduct, get_string_descriptor(handle, descriptor.iProduct).unwrap_or_default());
    debug!("  iSerialNumber: {:10} {}", descriptor.iSerialNumber, get_string_descriptor(handle, descriptor.iSerialNumber).unwrap_or_default());
    debug!("  bNumConfigurations: {:5}", descriptor.bNumConfigurations);
    debug!("  Speed: {:>25}\n", get_device_speed(speed));
}

pub(crate) fn get_descriptor_type(desc_type: u8) -> &'static str {
    match desc_type {
        ffi::LIBUSB_DT_DEVICE => "Device",
        ffi::LIBUSB_DT_CONFIG => "Configuration",
        ffi::LIBUSB_DT_STRING => "String",
        ffi::LIBUSB_DT_INTERFACE => "Interface",
        ffi::LIBUSB_DT_ENDPOINT => "Endpoint",
        ffi::LIBUSB_DT_BOS => "BOS",
        ffi::LIBUSB_DT_DEVICE_CAPABILITY => "Device Capability",
        ffi::LIBUSB_DT_HID => "HID",
        ffi::LIBUSB_DT_REPORT => "Report",
        ffi::LIBUSB_DT_PHYSICAL => "Physical",
        ffi::LIBUSB_DT_HUB => "HUB",
        ffi::LIBUSB_DT_SUPERSPEED_HUB => "Superspeed Hub",
        ffi::LIBUSB_DT_SS_ENDPOINT_COMPANION => "Superspeed Endpoint Companion",
        _ => "unknown type"
    }
}

pub(crate) fn get_bcd_version(bcd_version: u16) -> String {
    let digit1 = (bcd_version & 0xF000) >> 12;
    let digit2 = (bcd_version & 0x0F00) >> 8;
    let digit3 = (bcd_version & 0x00F0) >> 4;
    let digit4 = (bcd_version & 0x000F) >> 0;

    if digit1 > 0 {
        format!("{}{}.{}{}", digit1, digit2, digit3, digit4)
    }
    else {
        format!("{}.{}{}", digit2, digit3, digit4)
    }
}

pub(crate) fn get_class_type(class: u8) -> &'static str {
    match class {
        ffi::LIBUSB_CLASS_PER_INTERFACE       => "(Defined at Interface level)",
        ffi::LIBUSB_CLASS_AUDIO               => "Audio",
        ffi::LIBUSB_CLASS_COMM                => "Comm",
        ffi::LIBUSB_CLASS_HID                 => "HID",
        ffi::LIBUSB_CLASS_PHYSICAL            => "Physical",
        ffi::LIBUSB_CLASS_PRINTER             => "Printer",
        ffi::LIBUSB_CLASS_IMAGE               => "Image",
        ffi::LIBUSB_CLASS_MASS_STORAGE        => "Mass Storage",
        ffi::LIBUSB_CLASS_HUB                 => "Hub",
        ffi::LIBUSB_CLASS_DATA                => "Data",
        ffi::LIBUSB_CLASS_SMART_CARD          => "Smart Card",
        ffi::LIBUSB_CLASS_CONTENT_SECURITY    => "Content Security",
        ffi::LIBUSB_CLASS_VIDEO               => "Video",
        ffi::LIBUSB_CLASS_PERSONAL_HEALTHCARE => "Personal Healthcare",
        ffi::LIBUSB_CLASS_DIAGNOSTIC_DEVICE   => "Diagnostic Device",
        ffi::LIBUSB_CLASS_WIRELESS            => "Wireless",
        ffi::LIBUSB_CLASS_APPLICATION         => "Application",
        ffi::LIBUSB_CLASS_VENDOR_SPEC         => "Vendor Specific",
        _ => ""
    }
}

pub(crate) fn get_string_descriptor(handle: *mut ffi::libusb_device_handle, desc_index: u8) -> Option<String> {
    if handle.is_null() || desc_index == 0 {
        return None
    }

    let mut vec = Vec::<u8>::with_capacity(256);
    let ptr = (&mut vec[..]).as_mut_ptr();

    let len = unsafe { ffi::libusb_get_string_descriptor_ascii(handle, desc_index, ptr as *mut c_uchar, vec.capacity() as c_int) };

    if len > 0 {
        unsafe { vec.set_len(len as usize) };

        match String::from_utf8(vec) {
            Ok(s) => Some(s),
            Err(_) => None
        }
    } else {
        None
    }
}

pub(crate) fn get_device_speed(speed: c_int) -> &'static str {
    match speed {
        ffi::LIBUSB_SPEED_SUPER       => "5000 Mbps",
        ffi::LIBUSB_SPEED_HIGH        => " 480 Mbps",
        ffi::LIBUSB_SPEED_FULL        => "  12 Mbps",
        ffi::LIBUSB_SPEED_LOW         => " 1.5 Mbps",
        ffi::LIBUSB_SPEED_UNKNOWN     => "(unknown)",
        _ => "what's an odd usb speed value?",
    }
}
