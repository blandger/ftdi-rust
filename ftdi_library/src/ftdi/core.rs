#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(const_err)]
#![allow(unused_imports)]

use libusb_sys as ffi;
use libc;
use libc::{c_int,c_uchar};
use crate::ftdi::constants::{*};
use crate::ftdi::eeprom::ftdi_eeprom;
use std::sync::{Arc, Mutex};
use std::mem;
use std::io;
use snafu::{ensure, Backtrace, ErrorCompat, ResultExt, Snafu};
use log::{debug, info, error};

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
    interface: bool,   /* 0 or 1 */
    /// FT2232C index number: 1 or 2
    index: i32,       /* 1 or 2 */
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

    fn get_error(err: c_int) -> &'static str {
        match err {
            ffi::LIBUSB_SUCCESS             => "success",
            ffi::LIBUSB_ERROR_IO            => "I/O error",
            ffi::LIBUSB_ERROR_INVALID_PARAM => "invalid parameter",
            ffi::LIBUSB_ERROR_ACCESS        => "access denied",
            ffi::LIBUSB_ERROR_NO_DEVICE     => "no such device",
            ffi::LIBUSB_ERROR_NOT_FOUND     => "entity not found",
            ffi::LIBUSB_ERROR_BUSY          => "resource busy",
            ffi::LIBUSB_ERROR_TIMEOUT       => "opteration timed out",
            ffi::LIBUSB_ERROR_OVERFLOW      => "overflow error",
            ffi::LIBUSB_ERROR_PIPE          => "pipe error",
            ffi::LIBUSB_ERROR_INTERRUPTED   => "system call interrupted",
            ffi::LIBUSB_ERROR_NO_MEM        => "insufficient memory",
            ffi::LIBUSB_ERROR_NOT_SUPPORTED => "operation not supported",
            ffi::LIBUSB_ERROR_OTHER | _     => "other error"
        }
    }

    pub fn new() -> ftdi_context {
        debug!("start ftdi context creation...");
        let mut context: *mut ffi::libusb_context = unsafe { mem::MaybeUninit::uninit().assume_init() };
        debug!("ftdi context before init...");
        match unsafe { ffi::libusb_init(&mut context) } {
            0 => {
                debug!("ftdi context initialized - OK!");
            },
            e => {
                let error_msg = ftdi_context::get_error(e);
                error!("{}", error_msg);
                panic!("libusb_init() failed {}", error_msg)
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
        ftdi_context{
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
            writebuffer_chunksize: 4096,
            max_packet_size: 0,
            interface: false,
            index: 0,
            in_ep: 0,
            out_ep: 0,
            bitbang_mode: 0,
            eeprom: ftdi_eeprom,
            error_str: 0,
            module_detach_mode: ftdi_module_detach_mode::AUTO_DETACH_SIO_MODULE,
        }
    }
}

impl Drop for ftdi_context {
    fn drop(&mut self) {
        debug!("dropping ftdi context...");
        unsafe { ffi::libusb_exit(self.usb_ctx) };
    }
}


#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("Usb sys init error {}: {}", code, source))]
    UsbInit {
        code: i32,
        source: io::Error,
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
    // pub ftdi_device_list *next, // ???
    /// pointer to libusb's usb_device
    pub dev: *mut ffi::libusb_device,
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
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct ftdi_version_info {
    pub major: i32,
    pub minor: i32,
    pub micro: i32,
    pub version_str: *const char,
    pub snapshot_str: *const char,
}
