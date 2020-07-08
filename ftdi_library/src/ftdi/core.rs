#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(const_err)]
#![allow(unused_imports)]

use libusb_sys as ffi;
use libc::{c_int,c_uchar};
use crate::ftdi::constants::{*};
use crate::ftdi::eeprom::ftdi_eeprom;
use std::sync::{Arc, Mutex};
use std::{mem::{MaybeUninit}, slice, io};
use snafu::{ensure, Backtrace, ErrorCompat, ResultExt, Snafu};
use log::{debug, info, error};
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
        debug!("start ftdi context creation...");
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
                usb_dev: None,
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
        self.readbuffer_chunksize = self.check_return_size();
        self.readbuffer_chunksize
    }

    /// We can't set readbuffer_chunksize larger than MAX_BULK_BUFFER_LENGTH,
    /// which is defined in libusb-1.0.  Otherwise, each USB read request will
    /// be divided into multiple URBs.  This will cause issues on Linux kernel
    /// older than 2.6.32.
    #[cfg(target_os = "linux")]
    fn check_return_size(&self) -> u32 {
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

}

impl Drop for ftdi_context {
    fn drop(&mut self) {
        debug!("closing ftdi context...");
        match self.usb_dev {
            Some(usb_device) => {
                debug!("closing ftdi \'usb device handler\' context...");
                unsafe {ffi::libusb_close(usb_device);}
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
    /// pointer to next entry
    pub ftdi_device_list: Vec<*mut ffi::libusb_device>, // ???
    // pointer to libusb's usb_device
    // pub dev: *mut ffi::libusb_device,
}
impl ftdi_device_list {
    pub fn new(/*&mut self, */ftdi: ftdi_context) -> Result<Self> {
        debug!("start ftdi device list creation...");
        let mut device_list: *const *mut ffi::libusb_device = unsafe {
            MaybeUninit::uninit().assume_init()
        };
        let devices_len = unsafe { ffi::libusb_get_device_list(ftdi.usb_ctx, &mut device_list) };
        if devices_len < 0 {
            let result = ftdi_context::get_usb_sys_init_error(devices_len as c_int);
            error!("{}", result);
            return Err(result);
        }
        debug!("found usb device quantity = {}", devices_len);
        let sys_device_list = unsafe { slice::from_raw_parts(device_list, devices_len as usize) };
        let mut new_device_list: Vec<*mut ffi::libusb_device> = Vec::with_capacity(devices_len as usize);
        for dev in sys_device_list {
            new_device_list.push(*dev);
            // display_device(dev);
        }
        let list = ftdi_device_list{ftdi_device_list: new_device_list};
        debug!("stored usb device quantity = {}", devices_len);
        Ok(list)
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
