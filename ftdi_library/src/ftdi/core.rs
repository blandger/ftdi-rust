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

/// brief Main context structure for all libftdi functions.
/// Do not access directly if possible.
// #[derive(Copy, Debug)]
#[repr(C)]
pub struct ftdi_context {
    /// USB specific
    /// libusb's context
    usb_ctx: *mut ffi::libusb_context,
    /// libusb's usb_dev_handle
    usb_dev: Option<*mut ffi::libusb_device_handle>,
    /// usb read timeout
    usb_read_timeout: i32,
    /// usb write timeout
    usb_write_timeout: i32,

    /// FTDI specific
    /// FTDI chip type
    r#type: ftdi_chip_type,
    /// baudrate
    baudrate: i32,
    /// bitbang mode state
    bitbang_enabled: u8 /*libc::c_char*/,
    /// pointer to read buffer for ftdi_read_data
    readbuffer: [u8; 256],
    /// read buffer offset
    readbuffer_offset: u32,
    /// number of remaining data in internal read buffer
    readbuffer_remaining: u32,
    /// read buffer chunk size
    readbuffer_chunksize: u32,
    /// write buffer chunk size
    writebuffer_chunksize: u32,
    /// maximum packet size. Needed for filtering modem status bytes every n packets.
    max_packet_size: u32,

    /// FTDI FT2232C requirements
    /// FT2232C interface number: 0 or 1
    interface: u8,   /* 0 or 1 */
    /// FT2232C index number: 1 or 2
    index: u8,       /* 1 or 2 */
    /// Endpoints */
    /// FT2232C end points: 1 or 2
    in_ep: i32,
    out_ep: i32,      /* 1 or 2 */

    /// Bitbang mode. 1: (default) Normal bitbang mode, 2: FT2232C SPI bitbang mode
     bitbang_mode: u8,

    /// Decoded eeprom structure
    eeprom: ftdi_eeprom,

    /// String representation of last error
    error_str: i8,

    /// Defines behavior in case a kernel module is already attached to the device
    module_detach_mode: ftdi_module_detach_mode,
}

impl ftdi_context {

    pub fn get_usb_sys_init_error(err: c_int) -> FtdiError {
        match err {
            ffi::LIBUSB_SUCCESS             => FtdiError::UsbInit{code: 0, message: "success".to_string()},
            ffi::LIBUSB_ERROR_IO            => FtdiError::UsbInit{code: -1, message: "I/O error".to_string()},
            ffi::LIBUSB_ERROR_INVALID_PARAM => FtdiError::UsbInit{code: -2, message: "invalid parameter".to_string()},
            ffi::LIBUSB_ERROR_ACCESS        => FtdiError::UsbInit{code: -3, message: "access denied".to_string()},
            ffi::LIBUSB_ERROR_NO_DEVICE     => FtdiError::UsbInit{code: -4, message: "no such device".to_string()},
            ffi::LIBUSB_ERROR_NOT_FOUND     => FtdiError::UsbInit{code: -5, message: "entity not found".to_string()},
            ffi::LIBUSB_ERROR_BUSY          => FtdiError::UsbInit{code: -6, message: "resource busy".to_string()},
            ffi::LIBUSB_ERROR_TIMEOUT       => FtdiError::UsbInit{code: -7, message: "operation timed out".to_string()},
            ffi::LIBUSB_ERROR_OVERFLOW      => FtdiError::UsbInit{code: -8, message: "overflow error".to_string()},
            ffi::LIBUSB_ERROR_PIPE          => FtdiError::UsbInit{code: -9, message: "pipe error".to_string()},
            ffi::LIBUSB_ERROR_INTERRUPTED   => FtdiError::UsbInit{code: -10, message: "system call interrupted".to_string()},
            ffi::LIBUSB_ERROR_NO_MEM        => FtdiError::UsbInit{code: -11, message: "insufficient memory".to_string()},
            ffi::LIBUSB_ERROR_NOT_SUPPORTED => FtdiError::UsbInit{code: -12, message: "operation not supported".to_string()},
            ffi::LIBUSB_ERROR_OTHER         => FtdiError::UsbInit{code: -99, message: "other error".to_string()},
            _                               => FtdiError::UsbInit{code: -1000, message: "unknown error".to_string()},
        }
    }

    pub fn new() -> Result<Self> {
        debug!("start \'new\' ftdi context creation...");
        let mut context: *mut ffi::libusb_context = unsafe { MaybeUninit::uninit().assume_init() };
        debug!("ftdi context before init...");
        match unsafe { ffi::libusb_init(&mut context) } {
            0 => {
                debug!("ftdi context initialized - OK!");
            },
            sys_error => {
                // Err(ftdi_context::get_error(e))
                let error_enum = ftdi_context::get_usb_sys_init_error(sys_error);
                error!("{}", error_enum);
                return Err(error_enum);
            }
        };

        let ftdi_eeprom = ftdi_eeprom {
            vendor_id: 0,
            product_id: 0,
            initialized_for_connected_device: false,
            self_powered: 0,
            remote_wakeup: 0,
            is_not_pnp: false,
            suspend_dbus7: 0,
            in_is_isochronous: false,
            out_is_isochronous: false,
            suspend_pull_downs: 0,
            use_serial: false,
            usb_version: 0,
            use_usb_version: 0,
            max_power: 0,
            manufacturer: [0;256],
            product: [0;256],
            serial: [0;256],
            channel_a_type: 0,
            channel_b_type: 0,
            channel_a_driver: 0,
            channel_b_driver: 0,
            channel_c_driver: 0,
            channel_d_driver: 0,
            channel_a_rs485enable: false,
            channel_b_rs485enable: false,
            channel_c_rs485enable: false,
            channel_d_rs485enable: false,
            cbus_function: [0i32; 10],
            high_current: 0,
            high_current_a: 0,
            high_current_b: 0,
            invert: 0,
            external_oscillator: 0,
            group0_drive: 0,
            group0_schmitt: 0,
            group0_slew: 0,
            group1_drive: 0,
            group1_schmitt: 0,
            group1_slew: 0,
            group2_drive: 0,
            group2_schmitt: 0,
            group2_slew: 0,
            group3_drive: 0,
            group3_schmitt: 0,
            group3_slew: 0,
            powersave: 0,
            clock_polarity: 0,
            data_order: 0,
            flow_control: 0,
            user_data_addr: 0,
            user_data_size: 0,
            user_data: [0;256],
            size: 0,
            chip: 0,
            buf: [0;256],
            release_number: 0,
        };
        debug!("ftdi context is DONE!");
        Ok(
            ftdi_context {
                usb_ctx: context,
                usb_dev: None, // usb device to be assigned if it's found
                usb_read_timeout: 5000,
                usb_write_timeout: 5000,
                r#type: ftdi_chip_type::TYPE_BM,
                baudrate: -1,
                bitbang_enabled: 0,
                readbuffer: [0;256],
                readbuffer_offset: 0,
                readbuffer_remaining: 0,
                readbuffer_chunksize: 0,
                writebuffer_chunksize: READ_BUFFER_CHUNKSIZE,
                max_packet_size: 0,
                interface: 0,
                index: 0,
                in_ep: 0,
                out_ep: 0,
                bitbang_mode: 0,
                eeprom: ftdi_eeprom,
                error_str: 0,
                module_detach_mode: ftdi_module_detach_mode::AUTO_DETACH_SIO_MODULE,
            }
        )
    }

    pub fn set_interface_type(&mut self, interface_type: ftdi_interface) {
        debug!("set interface type \'{:?}\' to ftdi context", interface_type);
        match interface_type {
            ftdi_interface::INTERFACE_ANY | ftdi_interface::INTERFACE_A => {
                self.interface = 0;
                self.index     = ftdi_interface::INTERFACE_A.into();
                self.in_ep     = 0x02;
                self.out_ep    = 0x81;
            }
            ftdi_interface::INTERFACE_B => {
                self.interface = 1;
                self.index     = ftdi_interface::INTERFACE_B.into();
                self.in_ep     = 0x04;
                self.out_ep    = 0x83;
            }
            ftdi_interface::INTERFACE_C => {
                self.interface = 2;
                self.index     = ftdi_interface::INTERFACE_C.into();
                self.in_ep     = 0x06;
                self.out_ep    = 0x85;
            }
            ftdi_interface::INTERFACE_D => {
                self.interface = 3;
                self.index     = ftdi_interface::INTERFACE_D.into();
                self.in_ep     = 0x08;
                self.out_ep    = 0x87;
            }
        }
        self.bitbang_mode = 1; /* when bitbang is enabled this holds the number of the mode  */
    }

    pub  fn ftdi_read_data_set_chunksize(&mut self) -> u32 {
        self.readbuffer_offset = 0;
        self.readbuffer_remaining = 0;
        self.readbuffer_chunksize = self.check_retur_buffer_size();
        self.readbuffer_chunksize
    }

    /// We can't set readbuffer_chunksize larger than MAX_BULK_BUFFER_LENGTH,
    /// which is defined in libusb-1.0.  Otherwise, each USB read request will
    /// be divided into multiple URBs.  This will cause issues on Linux kernel
    /// older than 2.6.32.
    #[cfg(target_os = "linux")]
    fn check_retur_buffer_size(&self) -> u32 {
        let linux_kernel_version = version();
        match linux_kernel_version {
            Ok(version) if (version.major <= 2 && version.minor <= 6 && version.patch <= 32 ) => {
                READ_BUFFER_CHUNKSIZE_LINUX_LOW_KERNEL
            }
            _ => {
                READ_BUFFER_CHUNKSIZE
            }
        }
    }

    // And this function only gets compiled if the target OS is *not* linux
    #[cfg(not(target_os = "linux"))]
    fn check_return_size() -> u32 {
        READ_BUFFER_CHUNKSIZE
    }

    pub fn ftdi_usb_open_desc_index(&mut self, vendor: u16, product: u16,
                                    description: Option<&str>,
                                    serial: Option<&str>,
                                    index: usize) -> Result<&Self> {
        debug!("start \'ftdi_usb_open_desc_index\' ...");
        let mut device_list = ftdi_device_list::new(self)?;

        let sys_device_list = unsafe { slice::from_raw_parts(
            device_list.system_device_list.unwrap(), device_list.number_found_devices) };
        // let mut new_device_list: Vec<*mut ffi::libusb_device> = Vec::with_capacity(devices_len as usize);
        let mut usb_dev_index = 0;
        for dev in sys_device_list {
            // new_device_list.push(*dev);
            let speed = unsafe { ffi::libusb_get_device_speed(*dev) };
            let mut descriptor = unsafe { MaybeUninit::uninit().assume_init() };
            let has_descriptor = match unsafe { ffi::libusb_get_device_descriptor(*dev, &mut descriptor) } {
                0 => {
                    true
                },
                _err => {
                    error!("{}", FtdiError::UsbInit{code: -13, message: "libusb_get_device_descriptor() failed".to_string()});
                    false
                },
            };
            let mut handle: *mut ffi::libusb_device_handle = ptr::null_mut();
            if has_descriptor {
                info!("USB ID [{:?}] : {:04x}:{:04x}", usb_dev_index, descriptor.idVendor, descriptor.idProduct);
                print_debug_device_descriptor(handle, &descriptor, speed);
                // extract all usb devices OR only specified by vendor and product ids
                if vendor >= 0 && product >= 0 && descriptor.idVendor == vendor && descriptor.idProduct == product {
                    if unsafe { ffi::libusb_open(*dev, &mut handle) } < 0 {
                        warn!("Couldn't open device [{:?}], some information will be missing", usb_dev_index);
                        handle = ptr::null_mut();
                    } else {
                        debug!("found FTDI usb device by index = [{}]", usb_dev_index);
                        self.usb_dev = Some(handle); // assign found FTDI device
                    }
                }
                usb_dev_index += 1;
            }
        }
        // let list = ftdi_device_list{ftdi_device_list: new_device_list, system_device_list: Some(device_list)};
        debug!("stored usb device quantity = [{}]", device_list.number_found_devices);
        Ok(self)
    }

}

impl Drop for ftdi_context {
    fn drop(&mut self) {
        debug!("closing ftdi context...");
        match self.usb_dev {
            Some(usb_device) => {
                debug!("closing ftdi \'usb device handler\' context...");
                unsafe {ffi::libusb_close(usb_device);}
                self.usb_dev = None;
            }
            None => {
                debug!("NO ftdi \'usb device handler\' to close...");
            }
        }
        debug!("before usb context exit...");
        unsafe { ffi::libusb_exit(self.usb_ctx) };
        debug!("closing ftdi context is DONE!");
    }
}

type Result<T, E = FtdiError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
pub enum FtdiError {
    #[snafu(display("USB SYS INIT: {} - {}", code, message))]
    UsbInit {
        code: i32,
        message: String,
    },
    #[snafu(display("USB SYS COMMAND: {} - {}", code, message))]
    UsbCommandError {
        code: i32,
        message: String,
    }
}

// #[derive(Copy, Debug)]
#[repr(C)]
pub struct ftdi_transfer_control {
    pub completed: i32,
    pub buf: Vec<u8>,
    pub size: usize,
    pub offset: isize,
    pub ftdi: Arc<Mutex<ftdi_context>>,
    // pub transfer: ffi::libusb_transfer,
    pub transfer: Arc<Mutex<ffi::libusb_transfer>>,
}

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
        let mut device_list: *const *mut ffi::libusb_device = unsafe {
            MaybeUninit::uninit().assume_init()
        };
        let devices_len = unsafe { ffi::libusb_get_device_list(ftdi.usb_ctx, &mut device_list) };
        if devices_len < 0 {
            let result = FtdiError::UsbCommandError { code: -5, message: "libusb_get_device_list() failed".to_string() };
            error!("{}", result);
            return Err(result);
        }
        debug!("found total usb device(s) quantity = [{}]", devices_len);
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
        let mut device_list: *const *mut ffi::libusb_device = unsafe {
            MaybeUninit::uninit().assume_init()
        };
        let devices_len = unsafe { ffi::libusb_get_device_list(ftdi.usb_ctx, &mut device_list) };
        if devices_len < 0 {
            let result = FtdiError::UsbCommandError { code: -5, message: "libusb_get_device_list() failed".to_string() };
            error!("{}", result);
            return Err(result);
        }
        debug!("found total usb device(s) quantity = [{}]", devices_len);
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
                err => {
                    error!("{}", FtdiError::UsbCommandError{code: -6, message: "libusb_get_device_descriptor() failed".to_string()});
                    false
                },
            };
            let mut handle: *mut ffi::libusb_device_handle = ptr::null_mut();
            if has_descriptor {

                // extract usb devices only specified by vendor and product ids
                if ((vendor > 0 || product > 0) &&
                    descriptor.idVendor == vendor && descriptor.idProduct == product) ||
                    (!(vendor < 0 || product < 0) &&
                        (descriptor.idVendor == 0x403) && (descriptor.idProduct == 0x6001 || descriptor.idProduct == 0x6010
                        || descriptor.idProduct == 0x6011 || descriptor.idProduct == 0x6014
                        || descriptor.idProduct == 0x6015)) {
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


fn print_debug_device_descriptor(handle: *mut ffi::libusb_device_handle,
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
    debug!("  iManufacturer: {:10} {}", descriptor.iManufacturer, get_string_descriptor(handle, descriptor.iManufacturer).unwrap_or(String::new()));
    debug!("  iProduct: {:15} {}", descriptor.iProduct, get_string_descriptor(handle, descriptor.iProduct).unwrap_or(String::new()));
    debug!("  iSerialNumber: {:10} {}", descriptor.iSerialNumber, get_string_descriptor(handle, descriptor.iSerialNumber).unwrap_or(String::new()));
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

enum ftdi_cbus_func {
    CBUS_TXDEN = 0, CBUS_PWREN = 1, CBUS_RXLED = 2, CBUS_TXLED = 3, CBUS_TXRXLED = 4,
    CBUS_SLEEP = 5, CBUS_CLK48 = 6, CBUS_CLK24 = 7, CBUS_CLK12 = 8, CBUS_CLK6 =  9,
    CBUS_IOMODE = 0xa, CBUS_BB_WR = 0xb, CBUS_BB_RD = 0xc
}

enum ftdi_cbush_func {
    CBUSH_TRISTATE = 0, CBUSH_TXLED = 1, CBUSH_RXLED = 2, CBUSH_TXRXLED = 3, CBUSH_PWREN = 4,
    CBUSH_SLEEP = 5, CBUSH_DRIVE_0 = 6, CBUSH_DRIVE1 = 7, CBUSH_IOMODE = 8, CBUSH_TXDEN =  9,
    CBUSH_CLK30 = 10, CBUSH_CLK15 = 11, CBUSH_CLK7_5 = 12
}

enum ftdi_cbusx_func {
    CBUSX_TRISTATE = 0, CBUSX_TXLED = 1, CBUSX_RXLED = 2, CBUSX_TXRXLED = 3, CBUSX_PWREN = 4,
    CBUSX_SLEEP = 5, CBUSX_DRIVE_0 = 6, CBUSX_DRIVE1 = 7, CBUSX_IOMODE = 8, CBUSX_TXDEN =  9,
    CBUSX_CLK24 = 10, CBUSX_CLK12 = 11, CBUSX_CLK6 = 12, CBUSX_BAT_DETECT = 13,
    CBUSX_BAT_DETECT_NEG = 14, CBUSX_I2C_TXE = 15, CBUSX_I2C_RXF = 16, CBUSX_VBUS_SENSE = 17,
    CBUSX_BB_WR = 18, CBUSX_BB_RD = 19, CBUSX_TIME_STAMP = 20, CBUSX_AWAKE = 21
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct size_and_time {
    pub total_bytes: usize,
    /// seconds or milliseconds
    pub timeval: u128,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct progress {
    pub first: size_and_time,
    pub prev: size_and_time,
    pub current: size_and_time,
    pub total_time: f64,
    pub total_rate: f64,
    pub current_rate: f64,
}
type FTDIProgressInfo = progress;

/// Provide libftdi version information
/// major: Library major version
/// minor: Library minor version
/// micro: Currently unused, ight get used for hotfixes.
/// version_str: Version as (static) string
/// snapshot_str: Git snapshot version if known. Otherwise "unknown" or empty string.
#[derive(PartialEq, Eq, Debug)]
#[repr(C)]
pub struct ftdi_version_info {
    pub major: u8,
    pub minor: u8,
    pub micro: u8,
    pub version_str: String,
    pub snapshot_str: String,
}

impl ftdi_version_info {

    pub fn ftdi_get_library_version() -> ftdi_version_info  {
        ftdi_version_info {
            major: FTDI_MAJOR_VERSION,
            minor: FTDI_MINOR_VERSION,
            micro: FTDI_MICRO_VERSION,
            version_str: FTDI_VERSION_STRING.to_string(),
            snapshot_str: FTDI_SNAPSHOT_VERSION.to_string()
        }
    }
}
