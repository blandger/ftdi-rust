#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(const_err)]
#![allow(unused_imports)]

use libusb_sys as ffi;
use libc::{c_int,c_uchar, EPERM};
use crate::ftdi::constants::{*};
use crate::ftdi::eeprom::ftdi_eeprom;
use std::sync::{Arc, Mutex};
use std::{mem::{MaybeUninit}, slice, io, ptr};
use std::os::raw::{c_uint, c_ushort};
use std::any::Any;
use std::ptr::null;
use snafu::{ensure, Backtrace, ErrorCompat, ResultExt, Snafu};
use log::{debug, info, warn, error};
use linuxver::version;
use crate::ftdi::core::{FtdiError, Result};
use crate::ftdi::ftdi_device_list::{ftdi_device_list, print_debug_device_descriptor};
use crate::ftdi::constants::ftdi_module_detach_mode::{AUTO_DETACH_SIO_MODULE, AUTO_DETACH_REATACH_SIO_MODULE};

/// brief Main context structure for all libftdi functions.
/// Do not access directly if possible.
// #[derive(Copy, Debug)]
#[repr(C)]
pub struct ftdi_context {
    /// USB specific
    /// libusb's context
    pub usb_ctx: Option<*mut ffi::libusb_context>,
    // pub usb_ctx: MaybeUninit<*mut ffi::libusb_context>,
    /// libusb's usb_dev_handle
    pub usb_dev: Option<*mut ffi::libusb_device_handle>,
    /// usb read timeout
    pub usb_read_timeout: i32,
    /// usb write timeout
    pub usb_write_timeout: i32,

    /// FTDI specific
    /// FTDI chip type
    pub r#type: ftdi_chip_type,
    /// baudrate
    pub baudrate: i32,
    /// bitbang mode state
    pub bitbang_enabled: bool /*libc::c_char*/,
    /// pointer to read buffer for ftdi_read_data
    pub readbuffer: [u8; 256],
    /// read buffer offset
    pub readbuffer_offset: u32,
    /// number of remaining data in internal read buffer
    pub readbuffer_remaining: u32,
    /// read buffer chunk size
    pub readbuffer_chunksize: u32,
    /// write buffer chunk size
    pub writebuffer_chunksize: u32,
    /// maximum packet size. Needed for filtering modem status bytes every n packets.
    pub max_packet_size: u32,

    /// FTDI FT2232C requirements
    /// FT2232C interface number: 0 or 1
    pub interface: u8,   /* 0 or 1 */
    /// FT2232C index number: 1 or 2
    pub index: u8,       /* 1 or 2 */
    /// Endpoints */
    /// FT2232C end points: 1 or 2
    pub in_ep: i32,
    pub out_ep: i32,      /* 1 or 2 */

    /// Bitbang mode. 1: (default) Normal bitbang mode, 2: FT2232C SPI bitbang mode
    pub bitbang_mode: u8,

    /// Decoded eeprom structure
    pub eeprom: ftdi_eeprom,

    /// String representation of last error
    pub error_str: i8,

    /// Defines behavior in case a kernel module is already attached to the device
    pub module_detach_mode: ftdi_module_detach_mode,
}

impl ftdi_context {
    /// Helper functiona to convert USB system error code into FtdiError enum
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
    /// Allocate and initialize a new ftdi_context.
    ///
    /// ```rust,no_run
    ///use ::ftdi_library::ftdi::ftdi_context::ftdi_context;
    ///
    ///  let ftdi_context = ftdi_context::new();
    ///     match ftdi_context {
    ///         Ok(ftdi) => {
    ///             // use ftdi instance
    ///             println!("ftdi is OK, index = {}", ftdi.index);
    ///         },
    ///         Err(internal_error) => {
    ///             println!("{:?}", internal_error);
    ///         },
    ///     }
    /// ```
    /// or using without match
    ///
    /// ```rust,no_run
    ///use ::ftdi_library::ftdi::ftdi_context::ftdi_context;
    ///use ::ftdi_library::ftdi::core::{FtdiError};
    ///
    ///fn main() -> Result<(), FtdiError> {
    ///    let mut ftdi = ftdi_context::new()?;
    ///    Ok(())
    ///}
    /// ```
    pub fn new() -> Result<Self> {
        debug!("start \'new\' ftdi context creation...");
        // let mut context: MaybeUninit<*mut ffi::libusb_context> = unsafe { MaybeUninit::uninit().assume_init() };
        let mut context: *mut ffi::libusb_context = unsafe { MaybeUninit::uninit().assume_init() };
        // let _: [MaybeUninit<bool>; 5] = unsafe {
        //     MaybeUninit::uninit().assume_init()
        // };
        debug!("ftdi context before init...");
        match unsafe { ffi::libusb_init(&mut context/*.as_mut_ptr()*/) } {
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
                usb_ctx: Some(context),
                usb_dev: None, // usb device to be assigned if it's found
                usb_read_timeout: 5000,
                usb_write_timeout: 5000,
                r#type: ftdi_chip_type::TYPE_BM,
                baudrate: -1,
                bitbang_enabled: false,
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
                module_detach_mode: AUTO_DETACH_SIO_MODULE,
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
        self.readbuffer_chunksize = self.check_return_buffer_size();
        self.readbuffer_chunksize
    }

    /// We can't set read_buffer_chunksize larger than MAX_BULK_BUFFER_LENGTH,
    /// which is defined in libusb-1.0.  Otherwise, each USB read request will
    /// be divided into multiple URBs.  This will cause issues on Linux kernel
    /// older than 2.6.32.
    #[cfg(target_os = "linux")]
    fn check_return_buffer_size(&self) -> u32 {
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

    /// Opens the first device with a given vendor and product ids.
    // ftdi_context should be previously initialized otherwise return error.
    /// vendor is Vendor ID value
    /// product is Product ID value
    /// return same as ftdi_usb_open_desc()
    pub fn ftdi_usb_open(&mut self, vendor: u16, product: u16) -> Result<&Self> {
        ftdi_context::ftdi_usb_open_desc(self, vendor, product, None, None)
    }

    /// Opens the first device with a given, vendor id, product id,
    ///  description and serial.
    ///
    /// param vendor is Vendor ID value
    /// param product is Product ID value
    /// param description is Description to search for. Use NONE if not needed
    /// param serial is Serial to search for. Use NONE if not needed.
    pub fn ftdi_usb_open_desc(&mut self, vendor: u16, product: u16,
                              description: Option<String>,
                              serial: Option<String>) -> Result<&Self> {
        ftdi_context::ftdi_usb_open_desc_index(self, vendor, product, description, serial, 0)
    }

    // And this function only gets compiled if the target OS is *not* linux
    #[cfg(not(target_os = "linux"))]
    fn check_return_size() -> u32 {
        READ_BUFFER_CHUNKSIZE
    }

    /// Opens the index-th device with a given, vendor id, product id,
    ///  description and serial.
    ///
    ///  param vendor Vendor ID
    ///  param product Product ID
    ///  param description is Description to search for. Use None if not needed.
    ///  param serial is Serial to search for. Use None if not needed.
    /// param index Number of matching device to open if there are more than one, starts with 0.
    pub fn ftdi_usb_open_desc_index(&mut self, vendor: u16, product: u16,
                                    description: Option<String>,
                                    serial: Option<String>,
                                    mut index: usize) -> Result<&Self> {
        debug!("start \'ftdi_usb_open_desc_index\' ...");
        let device_list = ftdi_device_list::new(self)?;

        let sys_device_list = unsafe { slice::from_raw_parts(
            device_list.system_device_list.unwrap(), device_list.number_found_devices) };
        // let mut new_device_list: Vec<*mut ffi::libusb_device> = Vec::with_capacity(devices_len as usize);
        let mut usb_dev_index = 0;
        for dev in sys_device_list {

            let speed = unsafe { ffi::libusb_get_device_speed(*dev) };
            let mut descriptor = unsafe { MaybeUninit::uninit().assume_init() };
            let mut handle: *mut ffi::libusb_device_handle = ptr::null_mut();

            let has_descriptor = match unsafe { ffi::libusb_get_device_descriptor(*dev, &mut descriptor) } {
                0 => {
                    true
                },
                _err => {
                    error!("{}", FtdiError::UsbInit{code: -13, message: "libusb_get_device_descriptor() failed".to_string()});
                    false
                },
            };
            if has_descriptor {
                info!("USB ID [{:?}] : {:04x}:{:04x}", usb_dev_index, descriptor.idVendor, descriptor.idProduct);
                // print_debug_device_descriptor(handle, &descriptor, speed);
                // extract all usb devices OR only specified by vendor and product ids
                if vendor > 0 && product > 0 && descriptor.idVendor == vendor && descriptor.idProduct == product {
                    if unsafe { ffi::libusb_open(*dev, &mut handle) } < 0 {
                        warn!("Couldn't open device [{:?}], some information will be missing", usb_dev_index);
                    } else {
                        debug!("found FTDI usb device by index = [{}]", usb_dev_index);
                        print_debug_device_descriptor(handle, &descriptor, speed);
                        // self.usb_dev = Some(handle); // assign found FTDI device
                        let product_descriptor =
                            super::ftdi_device_list::get_string_descriptor(handle, descriptor.iProduct);
                        if description != None && product_descriptor != None && !description.eq(&product_descriptor.into()) {
                            if !handle.is_null() {
                                unsafe { ffi::libusb_close(handle) };
                            }
                            continue; // skip device
                        }
                        let serial_number =
                            super::ftdi_device_list::get_string_descriptor(handle, descriptor.iSerialNumber);
                        if serial != None && serial_number != None && !serial.eq(&serial_number) {
                            if !handle.is_null() {
                                unsafe { ffi::libusb_close(handle) };
                            }
                            continue; // skip device
                        }
                    }
                }
                usb_dev_index += 1;
            }
            if index > 0 {
                index -= index;
                if !handle.is_null() {
                    unsafe { ffi::libusb_close(handle) };
                }
            }
        }
        debug!("stored usb device quantity = [{}]", device_list.number_found_devices);
        Ok(self)
    }

    fn ftdi_read_chipid_shift(value: u32) -> u32 {
        ((value & 1) << 1) |
            ((value & 2) << 5) |
            ((value & 4) >> 2) |
            ((value & 8) << 4) |
            ((value & 16) >> 1) |
            ((value & 32) >> 1) |
            ((value & 64) >> 4) |
            ((value & 128) >> 2)
    }

    /// Read the FTDIChip-ID from R-type devices
    /// ftdi_context should be initialized previously
    /// return FTDIChip-ID value
    pub fn ftdi_read_chipid(&self) -> Result<u16> {
        debug!("start \'ftdi_read_chipid\' ...");
        if self.usb_dev == None {
            let error = FtdiError::UsbInit { code: -2, message: "USB device unavailable".to_string() };
            error!("{}", error);
            return Err(error);
        }
        let mut a: c_uchar = 0 as c_uchar;
        let mut b: c_uchar = 0 as c_uchar;
        let control_transfer_result_1 = unsafe {
            ffi::libusb_control_transfer(
                self.usb_dev.unwrap(),
                FTDI_DEVICE_IN_REQTYPE,
                SIO_READ_EEPROM_REQUEST,
                0, 0x43, &mut a, 2,
                self.usb_read_timeout as c_uint)
        };
        debug!("control_transfer_result_1 = {}", control_transfer_result_1);
        if control_transfer_result_1 == 2 {
            a = ((((a as u16) << 8) as u16) | ((a as u16) >> 8) as u16) as u8;
            let control_transfer_result_2 = unsafe {
                ffi::libusb_control_transfer(
                    self.usb_dev.unwrap(),
                    FTDI_DEVICE_IN_REQTYPE,
                    SIO_READ_EEPROM_REQUEST,
                    0, 0x44,&mut b, 2,
                    self.usb_read_timeout as c_uint)
            };
            debug!("control_transfer_result_2 = {}", control_transfer_result_2);
            if control_transfer_result_2 == 2 {
                // b = b << 8 | b >> 8; // old C code
                b = u16::from(u16::from(b) << 8 | u16::from(b) >> 8) as u8;
                // a = (a << 16) | (b & 0xFFFF); // old C code
                a = ((u32::from(a) << 16) | (u32::from(b) & 0xFFFF)) as u8;
                a = (ftdi_context::ftdi_read_chipid_shift(a as u32)
                    | ftdi_context::ftdi_read_chipid_shift((u32::from(a) >> 8) as u32) << 8
                    | ftdi_context::ftdi_read_chipid_shift((u32::from(a) >> 16) as u32) << 16
                    | ftdi_context::ftdi_read_chipid_shift((u32::from(a) >> 24) as u32) << 24) as u8;
                let chipid: u32 = ((a as u32) ^ (0xa5f0f7d1 as u32)) as u32;
                info!("Read ChipId = {}", chipid);
                return Ok(chipid as u16);
            } else {
                debug!("control_transfer_result_2 returned result = {}", control_transfer_result_2);
            }
        } else {
            debug!("control_transfer_result_1 returned result = {}", control_transfer_result_1);
        }
        let error = FtdiError::UsbCommandError { code: -1, message: "read of FTDIChip-ID failed".to_string() };
        Err(error)
    }

    /// Return device ID strings from the usb device.
    ///
    /// Returns device parameters as tuple of optional String: manufacturer, description and serial.
    /// They may be None if they were not fetched.
    /// Note - Use this function only in combination with ftdi_usb_find_all()
    ///    as it closes the internal "usb_dev" after use.
    /// param dev libusb usb_dev to use
    pub fn ftdi_usb_get_strings(&mut self, dev: *const *mut ffi::libusb_device)
        -> Result<(Option<String>, Option<String>, Option<String>)> {
        debug!("start \'ftdi_usb_get_strings\' ...");
        if self.usb_dev == None {
            let mut handle: *mut ffi::libusb_device_handle = ptr::null_mut();
            if unsafe { ffi::libusb_open(dev.cast(), &mut handle) } < 0 {
                warn!("Couldn't open device [{:?}], some information will be missing", dev.type_id());
                let error = FtdiError::UsbInit { code: -4, message: "libusb_open() failed".to_string() };
                error!("{}", error);
                return Err(error);
            }
            self.usb_dev = Some(handle);
            self.ftdi_usb_get_strings2(handle)
        } else {
            self.ftdi_usb_get_strings2(self.usb_dev.unwrap())
        }
    }
    /// Return device ID strings from the usb device.
    ///
    /// The parameter's manufacturer, description and serial may be None
    /// This version only closes the device if it was opened by it.
    fn ftdi_usb_get_strings2(&self, device_handle: *mut ffi::libusb_device_handle)
                             -> Result<(Option<String>, Option<String>, Option<String>)> {
        debug!("start \'ftdi_usb_get_strings\' ...");
        let mut descriptor = unsafe { MaybeUninit::uninit().assume_init() };
        let has_descriptor = match unsafe { ffi::libusb_get_device_descriptor(device_handle.cast(), &mut descriptor) } {
            0 => {
                true
            },
            _err => {
                error!("{}", FtdiError::UsbInit{code: -13, message: "libusb_get_device_descriptor() failed".to_string()});
                false
            },
        };
        if has_descriptor {
            info!("USB ID : {:04x} : {:04x} : {}", descriptor.idVendor, descriptor.idProduct, descriptor.iSerialNumber);
            print_debug_device_descriptor(device_handle, &descriptor, 0);

            let manufacturer_descriptor =
                super::ftdi_device_list::get_string_descriptor(device_handle, descriptor.iManufacturer);
            let product_descriptor =
                super::ftdi_device_list::get_string_descriptor(device_handle, descriptor.iProduct);
            let serial_number =
                super::ftdi_device_list::get_string_descriptor(device_handle, descriptor.iSerialNumber);
            return Ok( (manufacturer_descriptor, product_descriptor, serial_number) );
        } else {
            debug!("No usb description fetched for device");
        }
        Ok( (None, None, None) )
    }

    /// Opens a ftdi device given by an usb_device.
    //
    //  param dev libusb usb_dev to use
    pub fn ftdi_usb_open_dev(&mut self, dev: *const *mut ffi::libusb_device) -> Result<()> {
        debug!("start \'ftdi_usb_open_dev\' ...");
        // check ftdi context
        if self.usb_ctx == None {
            let error = FtdiError::UsbInit {code: -8, message: "ftdi context is not initialized previously".to_string()};
            error!("{}", error);
            return Err(error);
        }

        let mut handle: *mut ffi::libusb_device_handle = ptr::null_mut();
        if unsafe { ffi::libusb_open(dev.cast(), &mut handle ) } < 0 {
            warn!("Couldn't open device [{:?}], some information will be missing", dev.type_id());
            let error = FtdiError::UsbInit { code: -4, message: "libusb_open() failed".to_string() };
            error!("{}", error);
            return Err(error);
        }
        // self.usb_dev = Some(handle);

        let mut descriptor: ffi::libusb_device_descriptor = unsafe { MaybeUninit::uninit().assume_init() };
        let configuraton0: *mut *const ffi::libusb_config_descriptor = unsafe { MaybeUninit::uninit().assume_init() };
        if unsafe { ffi::libusb_get_device_descriptor(handle.cast(), &mut descriptor) } < 0 {
            let error = FtdiError::UsbCommandError { code: -9, message: "libusb_get_device_descriptor() failed".to_string() };
            error!("{}", error);
            return Err(error);
        };
        if unsafe { ffi::libusb_get_config_descriptor(handle.cast(), 0, configuraton0) } < 0 {
            let error = FtdiError::UsbCommandError { code: -10, message: "libusb_get_config_descriptor() failed".to_string() };
            error!("{}", error);
            return Err(error);
        };
        let cfg0: c_int = unsafe { (*(*configuraton0)).bConfigurationValue as c_int};
        unsafe { ffi::libusb_free_config_descriptor(*configuraton0) };

        let mut detach_errno = 0;
        let cfg: *mut c_int = 0 as *mut c_int;
        // let mut cfg0:c_int = 0;
        // Try to detach ftdi_sio kernel module.
        //
        // The return code is kept in a separate variable and only parsed
        // if usb_set_configuration() or usb_claim_interface() fails as the
        // detach operation might be denied and everything still works fine.
        // Likely scenario is a static ftdi_sio kernel module.
        if self.module_detach_mode == AUTO_DETACH_SIO_MODULE {
            match unsafe { ffi::libusb_detach_kernel_driver(handle, self.interface as c_int) } {
                0 => {
                    debug!("libusb_detach_kernel_driver for \'AUTO_DETACH_SIO_MODULE\' - OK!")
                },
                sys_error => {
                    let error_enum = ftdi_context::get_usb_sys_init_error(sys_error);
                    error!("libusb_detach_kernel_driver for \'AUTO_DETACH_SIO_MODULE\' {}", error_enum);
                    detach_errno = sys_error
                }
            }
        } else if self.module_detach_mode == AUTO_DETACH_REATACH_SIO_MODULE {
            match unsafe { ffi::libusb_set_auto_detach_kernel_driver(handle, 1) } {
                0 => {
                    debug!("libusb_detach_kernel_driver for \'AUTO_DETACH_REATACH_SIO_MODULE\' - OK!")
                },
                sys_error => {
                    let error_enum = ftdi_context::get_usb_sys_init_error(sys_error);
                    error!("libusb_detach_kernel_driver for \'AUTO_DETACH_REATACH_SIO_MODULE\' {}", error_enum);
                    detach_errno = sys_error
                }
            }
        }
        if unsafe { ffi::libusb_get_configuration (handle, cfg as *mut c_int) } < 0 {
            let error = FtdiError::UsbInit { code: -12, message: "libusb_get_configuration() failed".to_string() };
            error!("{}", error);
            return Err(error);
        }
        if descriptor.bNumConfigurations > 0 && (cfg != cfg0 as *mut c_int) {
            if unsafe { ffi::libusb_set_configuration(handle, cfg0) }  < 0 {
                if detach_errno == EPERM {
                    let error = FtdiError::UsbCommandError { code: -8, message: "inappropriate permissions on device!".to_string() };
                    error!("{}", error);
                    return Err(error);
                } else {
                    let error = FtdiError::UsbCommandError { code: -8,
                        message: "unable to set usb configuration. Make sure the default FTDI driver is not in use".to_string() };
                    error!("{}", error);
                    return Err(error);
                }
            }
        }
        self.usb_dev = Some(handle);
        self.ftdi_usb_reset()?;

        // Try to guess chip type
        // Bug in the BM type chips: bcdDevice is 0x200 for serial == 0
        if descriptor.bcdDevice == 0x400 || (descriptor.bcdDevice == 0x200 && descriptor.iSerialNumber == 0) {
            self.r#type = ftdi_chip_type::TYPE_BM;
        } else if descriptor.bcdDevice == 0x200 {
            self.r#type = ftdi_chip_type::TYPE_AM;
        } else if descriptor.bcdDevice == 0x500 {
            self.r#type = ftdi_chip_type::TYPE_2232C;
        } else if descriptor.bcdDevice == 0x600 {
            self.r#type = ftdi_chip_type::TYPE_R;
        } else if descriptor.bcdDevice == 0x700 {
            self.r#type = ftdi_chip_type::TYPE_2232H;
        } else if descriptor.bcdDevice == 0x800 {
            self.r#type = ftdi_chip_type::TYPE_4232H;
        } else if descriptor.bcdDevice == 0x900 {
            self.r#type = ftdi_chip_type::TYPE_232H;
        } else if descriptor.bcdDevice == 0x1000 {
            self.r#type = ftdi_chip_type::TYPE_230X;
        } else {
            let error = FtdiError::UsbInit { code: -8, message: "Is it new 'ftdi_chip_type' ?? or type is not guessed".to_string() };
            error!("{}", error);
            return Err(error);
        }
        // Determine maximum packet size
        self.max_packet_size = self.ftdi_determine_max_packet_size()?;
        self.ftdi_set_baudrate(9600)?;
        Ok(())
    }

    /// Resets the ftdi device.
    fn ftdi_usb_reset(&mut self) -> Result<()> {
        if self.usb_dev == None {
            let error = FtdiError::UsbInit {code: -2, message: "USB device unavailable".to_string()};
            error!("{}", error);
            return Err(error);
        }
        let null_data_ptr: *mut c_uchar = ptr::null_mut::<c_uchar>();
        if unsafe {ffi::libusb_control_transfer(self.usb_dev.unwrap(),
                                                FTDI_DEVICE_OUT_REQTYPE,
                                                SIO_RESET_REQUEST,
                                                SIO_RESET_SIO as u16,
                                                self.index as u16, null_data_ptr,
                                                0,
                                                self.usb_write_timeout as c_uint)} < 0 {
            let error = FtdiError::UsbCommandError {code: -1, message: "FTDI reset failed".to_string()};
            error!("{}", error);
            return Err(error);
        }
        // Invalidate data in the readbuffer
        self.readbuffer_offset = 0;
        self.readbuffer_remaining = 0;
        Ok(())
    }

    /// Internal function to determine the maximum packet size.
    ///  Return Maximum packet size for this device
    fn ftdi_determine_max_packet_size(&mut self)  -> Result<u32> {
        if self.usb_dev == None {
            let error = FtdiError::UsbInit {code: -2, message: "USB device unavailable".to_string()};
            error!("{}", error);
            return Ok(64);
        }
        let mut packet_size: u32 = 0;
        // Determine maximum packet size. Init with default value.
        // New hi-speed devices from FTDI use a packet size of 512 bytes
        // but could be connected to a normal speed USB hub -> 64 bytes packet size.
        if self.r#type == ftdi_chip_type::TYPE_2232H || self.r#type  == ftdi_chip_type::TYPE_4232H
            || self.r#type == ftdi_chip_type::TYPE_232H {
            packet_size = 512;
        } else {
            packet_size = 64;
        }
        let mut descriptor: ffi::libusb_device_descriptor = unsafe { MaybeUninit::uninit().assume_init() };
        let configuraton0: *mut *const ffi::libusb_config_descriptor = unsafe { MaybeUninit::uninit().assume_init() };
        if unsafe { ffi::libusb_get_device_descriptor(self.usb_dev.unwrap().cast(), &mut descriptor) } < 0 {
            let error = FtdiError::UsbCommandError { code: -9, message: "libusb_get_device_descriptor() failed".to_string() };
            error!("{}", error);
            return Ok(packet_size);
        };
        if unsafe { ffi::libusb_get_config_descriptor(self.usb_dev.unwrap().cast(), 0, configuraton0) } < 0 {
            let error = FtdiError::UsbCommandError { code: -10, message: "libusb_get_config_descriptor() failed".to_string() };
            error!("{}", error);
            return Ok(packet_size);
        };
        if descriptor.bNumConfigurations > 0 {
            if self.interface < unsafe { (*(*configuraton0)).bNumInterfaces } {
                let local_interface = unsafe { (*(*configuraton0)).interface/*[self.interface]*/ };
                if unsafe { (*local_interface).num_altsetting } > 0  {
                    let local_descriptor = unsafe { (*local_interface).altsetting/*[0]*/ };
                    if unsafe { (*local_descriptor).bNumEndpoints } > 0 {
                        packet_size = unsafe { (*(*local_descriptor).endpoint)/*[0]*/.wMaxPacketSize as u32 };
                    }
                }
            }
        }
        unsafe { ffi::libusb_free_config_descriptor(*configuraton0) };
        Ok(packet_size)
    }

    /// Sets the chip baud rate
    ///
    /// param baudrate baud rate to set
    fn ftdi_set_baudrate(&mut self, mut baudrate: i32)  -> Result<()> {
        if self.usb_dev == None {
            let error = FtdiError::UsbInit {code: -2, message: "USB device unavailable".to_string()};
            error!("{}", error);
            return Err(error);
        }
        if self.bitbang_enabled {
            baudrate = baudrate * 4;
        }
        let value: i32 = 0; let index: i32 = 0;
        let actual_baudrate: i32 = self.ftdi_convert_baudrate(baudrate, &value, &index);
        if actual_baudrate <= 0 {
            let error = FtdiError::UsbCommonError {code: -1, message: "Silly baudrate <= 0.".to_string()};
            error!("{}", error);
            return Err(error);
        }
        // Check within tolerance (about 5%)
        let compute_result = if (actual_baudrate < baudrate) {
            actual_baudrate * 21 < baudrate * 20
        } else {
            baudrate * 21i32 < actual_baudrate * 20
        };
        if (actual_baudrate * 2 < baudrate /* Catch overflows */ )
            || compute_result {
            let error = FtdiError::UsbCommonError {code: -1, message: "Unsupported baudrate. \
                Note: bitbang baudrates are automatically multiplied by 4".to_string()};
            error!("{}", error);
            return Err(error);
        }
        let null_data_ptr: *mut c_uchar = ptr::null_mut::<c_uchar>();
        if unsafe {ffi::libusb_control_transfer(self.usb_dev.unwrap(),
                                                FTDI_DEVICE_OUT_REQTYPE,
                                                SIO_SET_BAUDRATE_REQUEST,
                                                value as u16,
                                                index as u16, null_data_ptr,
                                                0,
                                                self.usb_write_timeout as c_uint)} < 0 {
            let error = FtdiError::UsbCommandError {code: -2, message: "Setting new baudrate failed".to_string()};
            error!("{}", error);
            return Err(error);
        }
        self.baudrate = baudrate;
        Ok(())
    }

    const H_CLK: i32 = 120000000;
    const C_CLK: i32 =  48000000;

    /// ftdi_convert_baudrate returns nearest supported baud rate to that requested.
    //  Function is only used internally
    fn ftdi_convert_baudrate(&mut self, mut baudrate: i32, value: &i32, index: &i32) -> i32 {
        let mut best_baud = -1;
        let mut encoded_divisor: u32 = 0;
        if baudrate <= 0 {
            let error = FtdiError::UsbCommonError {code: -2, message: "Incorrect baudrate".to_string()};
            warn!("{}", error);
            return -1;
        }
        if (self.r#type == ftdi_chip_type::TYPE_2232H)
            || (self.r#type == ftdi_chip_type::TYPE_4232H)
            || (self.r#type == ftdi_chip_type::TYPE_232H) {
            if (baudrate * 10) > (ftdi_context::H_CLK / 0x3fff) {
                /* On H Devices, use 12 000 000 Baudrate when possible
               We have a 14 bit divisor, a 1 bit divisor switch (10 or 16)
               three fractional bits and a 120 MHz clock
               Assume AN_120 "Sub-integer divisors between 0 and 2 are not allowed" holds for
               DIV/10 CLK too, so /1, /1.5 and /2 can be handled the same*/
                best_baud = self.ftdi_to_clkbits(baudrate, ftdi_context::H_CLK, 10, encoded_divisor);
                encoded_divisor |= 0x20000; /* switch on CLK/10*/
            } else {
                best_baud = self.ftdi_to_clkbits(baudrate, ftdi_context::C_CLK, 16, encoded_divisor);
            }
        }
        unimplemented!()
    }

    fn ftdi_to_clkbits(&mut self, baudrate: i32, clk: i32, clk_div: i32, encoded_divisor: u32) -> i32 {
        unimplemented!()
    }

}

impl Drop for ftdi_context {
    fn drop(&mut self) {
        debug!("closing ftdi context...");
        match self.usb_dev {
            Some(usb_device) => {
                debug!("closing ftdi \'usb device handler\' context...");
                unsafe {ffi::libusb_close(usb_device);}
                unsafe {ffi::libusb_release_interface(usb_device, self.interface as c_int); }
                self.usb_dev = None;
            }
            None => {
                debug!("NO ftdi \'usb device handler\' to close...");
            }
        }
        if self.usb_ctx != None {
            debug!("before usb context exit...");
            unsafe { ffi::libusb_exit(self.usb_ctx.unwrap()) };
        }
        debug!("closing ftdi context is DONE!");
    }
}

