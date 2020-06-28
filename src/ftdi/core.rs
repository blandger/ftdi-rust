#![allow(non_camel_case_types)]
#![allow(dead_code)]

use libusb_sys as ffi;
use crate::ftdi::{ftdi_chip_type, ftdi_module_detach_mode};
use crate::ftdi::eeprom::ftdi_eeprom;
use std::sync::{Arc, Mutex};

/// brief Main context structure for all libftdi functions.
/// Do not access directly if possible.
// #[derive(Copy, Debug)]
#[repr(C)]
pub struct ftdi_context {
    /// USB specific
    /// libusb's context
    usb_ctx: Arc<Mutex<ffi::libusb_context>>,
    /// libusb's usb_dev_handle
    usb_dev: Arc<Mutex<ffi::libusb_device_handle>>,
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

    /// FTDI FT2232C requirecments
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
