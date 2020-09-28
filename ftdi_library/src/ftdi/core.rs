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
use crate::ftdi::ftdi_context::ftdi_context;


#[derive(Debug, Snafu)]
pub enum FtdiError {
    #[snafu(display("USB SYS INIT: error code: \'{}\', message: \'{}\'\n{}", code, message, backtrace))]
    UsbInit {
        code: i32,
        message: String,
        backtrace: Backtrace,
    },
    #[snafu(display("USB SYS COMMAND: error code: \'{}\', message: \'{}\'\n{}", code, message, backtrace))]
    UsbCommandError {
        code: i32,
        message: String,
        backtrace: Backtrace,
    },
    #[snafu(display("COMMON ERROR: error code: \'{}\', message: \'{}\'\n{}", code, message, backtrace))]
    UsbCommonError {
        code: i32,
        message: String,
        backtrace: Backtrace,
    }
}

pub type Result<T, E = FtdiError> = std::result::Result<T, E>;

// #[derive(Copy, Debug)]
#[repr(C)]
pub struct ftdi_transfer_control {
    pub completed: i32,
    pub buf: Vec<u8>,
    pub size: i32,
    pub offset: i32,
    pub ftdi: Arc<Mutex<ftdi_context>>,
    // pub ftdi: Option<*mut ffi::libusb_context>,
    pub transfer: ffi::libusb_transfer,
    // pub transfer: Arc<Mutex<ffi::libusb_transfer>>,
}
impl Default for ftdi_transfer_control {
    fn default() -> Self {
        let libusb_transfer_uninit = MaybeUninit::<ffi::libusb_transfer>::zeroed();
        let libusb_transfer = unsafe { libusb_transfer_uninit.assume_init() };
        ftdi_transfer_control {
            completed: 0,
            buf: Vec::new(),
            size: 0,
            offset: 0,
            ftdi: Arc::new(Mutex::new(ftdi_context::default())),
            // transfer: Arc::new(Mutex::new( libusb_transfer ))
            transfer: libusb_transfer
        }
    }
}
impl ftdi_transfer_control {
    pub fn new(ftdi: ftdi_context, buffer: &Vec<u8>) -> Self {
    // pub fn new(ftdi: ftdi_context, buffer: &Box<[u8]>) -> Self {
        let libusb_transfer_uninit = MaybeUninit::<ffi::libusb_transfer>::zeroed();
        let libusb_transfer = unsafe { libusb_transfer_uninit.assume_init() };
        ftdi_transfer_control {
            completed: 0,
            buf: buffer.to_vec(), // TODO: check if cloning is correct way here
            size: buffer.len() as i32,
            offset: 0,
            ftdi: Arc::new(Mutex::new(ftdi)),
            // transfer: Arc::new(Mutex::new( libusb_transfer ))
            transfer: libusb_transfer,
        }
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

#[macro_export]
macro_rules! scanf {
    ( $string:expr, $sep:expr, $( $x:ty ),+ ) => {{
        let mut iter = $string.split($sep);
        ($(iter.next().and_then(|word| word.parse::<$x>().ok()),)*)
    }}
}
