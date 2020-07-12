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
use crate::ftdi::ftdi_device_list::{ftdi_device_list, print_debug_device_descriptor};

/// brief Main context structure for all libftdi functions.
/// Do not access directly if possible.
// #[derive(Copy, Debug)]
#[repr(C)]
pub struct ftdi_context {
    /// USB specific
    /// libusb's context
    // pub usb_ctx: *mut ffi::libusb_context,
    pub usb_ctx: MaybeUninit<*mut ffi::libusb_context>,
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
    pub bitbang_enabled: u8 /*libc::c_char*/,
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

    pub fn new() -> Result<Self> {
        debug!("start \'new\' ftdi context creation...");
        let mut context: MaybeUninit<*mut ffi::libusb_context> = unsafe { MaybeUninit::uninit().assume_init() };
        // let _: [MaybeUninit<bool>; 5] = unsafe {
        //     MaybeUninit::uninit().assume_init()
        // };
        debug!("ftdi context before init...");
        match unsafe { ffi::libusb_init(context.as_mut_ptr()) } {
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
        let device_list = ftdi_device_list::new(self)?;

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
                if vendor > 0 && product > 0 && descriptor.idVendor == vendor && descriptor.idProduct == product {
                    if unsafe { ffi::libusb_open(*dev, &mut handle) } < 0 {
                        warn!("Couldn't open device [{:?}], some information will be missing", usb_dev_index);
                        // handle = ptr::null_mut();
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
        unsafe { ffi::libusb_exit(*self.usb_ctx.as_mut_ptr()) };
        debug!("closing ftdi context is DONE!");
    }
}

