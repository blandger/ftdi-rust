#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(const_err)]
#![allow(unused_imports)]

use libusb_sys as ffi;
use libc::{c_int,c_uchar};
use crate::ftdi::constants::{*};
use crate::ftdi::eeprom::ftdi_eeprom;
use std::sync::{Arc, Mutex};
use std::{mem::{MaybeUninit}, slice, io, ptr};
use snafu::{ensure, Backtrace, ErrorCompat, ResultExt, Snafu};
use log::{debug, info, warn, error};
use linuxver::version;
use crate::ftdi::core::{FtdiError, Result};
use crate::ftdi::ftdi_context::ftdi_context;

/// brief list of usb devices created by ftdi_usb_find_all()
pub struct ftdi_device_list {
    /// Vector keeps all devices and all are freed later
    pub ftdi_device_list: Vec<*mut ffi::libusb_device>,
    /// found ans stored number of devices.
    /// It equals to number of devices in vector
    pub number_found_devices: usize,
    /// pointer to libusb's usb_device
    pub system_device_list: Option<*const *mut ffi::libusb_device>,
}
impl ftdi_device_list {
    /// Creates usb device list for all available devices in system
    pub fn new(ftdi: &ftdi_context) -> Result<Self> {
        // fetch usb device list
        let (device_list, devices_len) = ftdi_device_list::get_usb_device_list_internal(ftdi)?;
        let sys_device_list = unsafe { slice::from_raw_parts(
            device_list, devices_len as usize) };
        let mut new_device_list: Vec<*mut ffi::libusb_device> = Vec::with_capacity(devices_len as usize);
        for dev in sys_device_list {
            new_device_list.push(*dev); // push any device
        }
        let list = ftdi_device_list{
            ftdi_device_list: new_device_list,
            number_found_devices: devices_len as usize,
            system_device_list: Some(device_list)};
        debug!("stored usb device quantity = {}", devices_len);
        Ok(list)
    }

    /// Finds all ftdi devices with given VID:PID on the usb bus. Creates a new
    ///  ftdi_device_list which is deallocated automatically after use and going out of scope.
    ///  With VID:PID 0:0, it searches for the default devices
    ///  (0x403:0x6001, 0x403:0x6010, 0x403:0x6011, 0x403:0x6014, 0x403:0x6015)
    //
    ///   \param ftdi is ftdi_context
    ///   devlist is stored in devices field 'ftdi_device_list' vector
    ///   \param vendor Vendor ID to search for
    ///   \param product Product ID to search for
    pub fn ftdi_usb_find_all(ftdi: &ftdi_context, vendor: u16, product: u16) -> Result<Self> {
        // fetch usb device list
        let (device_list, devices_len) = ftdi_device_list::get_usb_device_list_internal(ftdi)?;
        // make slice to internate over
        let sys_device_list = unsafe { slice::from_raw_parts(
            device_list, devices_len as usize) };
        let mut new_device_list: Vec<*mut ffi::libusb_device> = Vec::with_capacity(devices_len as usize);
        let mut usb_dev_index = 0;
        for dev in sys_device_list {

            let speed = unsafe { ffi::libusb_get_device_speed(*dev) };
            let mut descriptor = unsafe { MaybeUninit::uninit().assume_init() };
            let has_descriptor = match unsafe { ffi::libusb_get_device_descriptor(*dev, &mut descriptor) } {
                0 => {
                    true
                },
                _err => {
                    error!("{}", FtdiError::UsbCommandError{code: -6, message: "libusb_get_device_descriptor() failed".to_string()});
                    false
                },
            };
            let handle: *mut ffi::libusb_device_handle = ptr::null_mut();
            if has_descriptor {

                // extract usb devices only specified by vendor and product ids
                if (vendor > 0 || product > 0 &&
                    descriptor.idVendor == vendor && descriptor.idProduct == product) ||
                    !(vendor > 0 || product > 0) &&
                        (descriptor.idVendor == 0x403) && (descriptor.idProduct == 0x6001 || descriptor.idProduct == 0x6010
                        || descriptor.idProduct == 0x6011 || descriptor.idProduct == 0x6014
                        || descriptor.idProduct == 0x6015) {
                    info!("USB ID [{:?}] : {:04x}:{:04x}", usb_dev_index, descriptor.idVendor, descriptor.idProduct);
                    print_debug_device_descriptor(handle, &descriptor, speed);

                    unsafe { ffi::libusb_ref_device(*dev) };
                    new_device_list.push(*dev);

                }
                usb_dev_index += 1;
            }

        }
        let list = ftdi_device_list{
            ftdi_device_list: new_device_list,
            number_found_devices: devices_len as usize,
            system_device_list: None};
        unsafe { ffi::libusb_free_device_list(device_list,1); };
        debug!("stored usb device quantity = {}", devices_len);
        Ok(list)
    }

    /// Helper internal method to fetch usb device list.
    /// Return tuple with found list and found device quantity
    fn get_usb_device_list_internal(ftdi: &ftdi_context) -> Result< (*const *mut ffi::libusb_device, isize) > {
        let mut device_list: *const *mut ffi::libusb_device = unsafe {
            MaybeUninit::uninit().assume_init()
        };
        let devices_len = unsafe { ffi::libusb_get_device_list(ftdi.usb_ctx.assume_init(), &mut device_list) };
        if devices_len < 0 {
            let result = FtdiError::UsbCommandError { code: -5, message: "libusb_get_device_list() failed".to_string() };
            error!("{}", result);
            return Err(result);
        }
        debug!("found total usb device(s) quantity = [{}]", devices_len);
        Ok( (device_list, devices_len) )
    }
}
impl Drop for ftdi_device_list {
    fn drop(&mut self) {
        debug!("cleaning up ftdi_device_list...");
        self.number_found_devices = 0;
        for dev in &self.ftdi_device_list {
            unsafe { ffi::libusb_unref_device(*dev); }
        }
        self.ftdi_device_list.clear();
        if self.system_device_list != None {
            unsafe { ffi::libusb_free_device_list(self.system_device_list.unwrap(), 1) };
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
    debug!("  Speed: {:#8}\n", get_device_speed(speed));
}

fn get_descriptor_type(desc_type: u8) -> &'static str {
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

fn get_bcd_version(bcd_version: u16) -> String {
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

fn get_class_type(class: u8) -> &'static str {
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

fn get_string_descriptor(handle: *mut ffi::libusb_device_handle, desc_index: u8) -> Option<String> {
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
    }
    else {
        None
    }
}

fn get_device_speed(speed: c_int) -> &'static str {
    match speed {
        ffi::LIBUSB_SPEED_SUPER       => "5000 Mbps",
        ffi::LIBUSB_SPEED_HIGH        => " 480 Mbps",
        ffi::LIBUSB_SPEED_FULL        => "  12 Mbps",
        ffi::LIBUSB_SPEED_LOW         => " 1.5 Mbps",
        ffi::LIBUSB_SPEED_UNKNOWN | _ => "(unknown)"
    }
}
