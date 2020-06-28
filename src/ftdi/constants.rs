
use libusb_sys as ffi;

// Heavy stick to translate between opcode types
use std::mem::transmute;

/// FTDI chip type
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum ftdi_chip_type {
    TYPE_AM = 0,
    TYPE_BM = 1,
    TYPE_2232C = 2,
    TYPE_R = 3,
    TYPE_2232H = 4,
    TYPE_4232H = 5,
    TYPE_232H = 6,
    TYPE_230X = 7,
}
impl From<u8> for ftdi_chip_type {
    #[inline]
    fn from(b: u8) -> ftdi_chip_type {
        unsafe { transmute(b) }
    }
}

/// Parity mode for ftdi_set_line_property()
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum ftdi_parity_type {
    NONE = 0,
    ODD = 1,
    EVEN = 2,
    MARK = 3,
    SPACE = 4
}
impl From<u8> for ftdi_parity_type {
    #[inline]
    fn from(b: u8) -> ftdi_parity_type {
        unsafe { transmute(b) }
    }
}

/// Number of stop bits for ftdi_set_line_property()
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum ftdi_stopbits_type {
    STOP_BIT_1 = 0,
    STOP_BIT_15 = 1,
    STOP_BIT_2 = 2
}
impl From<u8> for ftdi_stopbits_type {
    #[inline]
    fn from(b: u8) -> ftdi_stopbits_type {
        unsafe { transmute(b) }
    }
}

/// Number of bits for ftdi_set_line_property()
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum ftdi_bits_type {
    BITS_7 = 7,
    BITS_8 = 8
}
impl From<u8> for ftdi_bits_type {
    #[inline]
    fn from(b: u8) -> ftdi_bits_type {
        unsafe { transmute(b) }
    }
}

/// Break type for ftdi_set_line_property2()
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum ftdi_break_type {
    BREAK_OFF = 0,
    BREAK_ON = 1
}
impl From<u8> for ftdi_break_type {
    #[inline]
    fn from(b: u8) -> ftdi_break_type {
        unsafe { transmute(b) }
    }
}

/// MPSSE bitbang modes
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum ftdi_mpsse_mode {
    ///< switch off bitbang mode, back to regular serial/FIFO
    BITMODE_RESET = 0x00,
    ///< classical asynchronous bitbang mode, introduced with B-type chips
    BITMODE_BITBANG = 0x01,
    ///< MPSSE mode, available on 2232x chips
    BITMODE_MPSSE = 0x02,
    ///< synchronous bitbang mode, available on 2232x and R-type chips
    BITMODE_SYNCBB = 0x04,
    ///< MCU Host Bus Emulation mode, available on 2232x chips
    BITMODE_MCU = 0x08,
    /// CPU-style fifo mode gets set via EEPROM
    ///< Fast Opto-Isolated Serial Interface Mode, available on 2232x chips
    BITMODE_OPTO = 0x10,
    ///< Bitbang on CBUS pins of R-type chips, configure in EEPROM before
    BITMODE_CBUS = 0x20,
    ///< Single Channel Synchronous FIFO mode, available on 2232H chips
    BITMODE_SYNCFF = 0x40,
    ///< FT1284 mode, available on 232H chips
    BITMODE_FT1284 = 0x80,
}
impl From<u8> for ftdi_mpsse_mode {
    #[inline]
    fn from(b: u8) -> ftdi_mpsse_mode {
        unsafe { transmute(b) }
    }
}

/// Port interface for chips with multiple interfaces
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum ftdi_interface {
    INTERFACE_ANY = 0,
    INTERFACE_A = 1,
    INTERFACE_B = 2,
    INTERFACE_C = 3,
    INTERFACE_D = 4
}
impl From<u8> for ftdi_interface {
    #[inline]
    fn from(b: u8) -> ftdi_interface {
        unsafe { transmute(b) }
    }
}

/// Automatic loading / unloading of kernel modules
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum ftdi_module_detach_mode {
    AUTO_DETACH_SIO_MODULE = 0,
    DONT_DETACH_SIO_MODULE = 1,
    AUTO_DETACH_REATACH_SIO_MODULE = 2
}
impl From<u8> for ftdi_module_detach_mode {
    #[inline]
    fn from(b: u8) -> ftdi_module_detach_mode {
        unsafe { transmute(b) }
    }
}

/* Shifting commands IN MPSSE Mode*/
/// Write TDI/DO on negative TCK/SK edge
pub const MPSSE_WRITE_NEG: u8 = 0x01;
/// Write bits, not bytes
pub const MPSSE_BITMODE: u8 = 0x02;
/// Sample TDO/DI on negative TCK/SK edge
pub const MPSSE_READ_NEG: u8 = 0x04;
/// LSB first
pub const MPSSE_LSB: u8 = 0x08;
/// Write TDI/DO
pub const MPSSE_DO_WRITE: u8 = 0x10;
/// Read TDO/DI
pub const MPSSE_DO_READ: u8 = 0x20;
/// Write TMS/CS
pub const MPSSE_WRITE_TMS: u8 = 0x40;

// FTDI MPSSE commands
pub const SET_BITS_LOW: u8 = 0x80;
///BYTE DATA
///BYTE Direction
pub const SET_BITS_HIGH: u8 = 0x82;
///BYTE DATA
///BYTE Direction
pub const GET_BITS_LOW: u8 = 0x81;
pub const GET_BITS_HIGH: u8 = 0x83;
pub const LOOPBACK_START: u8 = 0x84;
pub const LOOPBACK_END: u8 = 0x85;
pub const TCK_DIVISOR: u8 = 0x86;

/// H Type specific commands
pub const DIS_DIV_5: u8 = 0x8a;
pub const EN_DIV_5: u8 = 0x8b;
pub const EN_3_PHASE: u8 = 0x8c;
pub const DIS_3_PHASE: u8 = 0x8d;
pub const CLK_BITS: u8 = 0x8e;
pub const CLK_BYTES: u8 = 0x8f;
pub const CLK_WAIT_HIGH: u8 = 0x94;
pub const CLK_WAIT_LOW: u8 = 0x95;
pub const EN_ADAPTIVE: u8 = 0x96;
pub const DIS_ADAPTIVE: u8 = 0x97;
pub const CLK_BYTES_OR_HIGH: u8 = 0x9c;
pub const CLK_BYTES_OR_LOW: u8 = 0x9d;

/// FT232H specific commands
pub const DRIVE_OPEN_COLLECTOR: u8 = 0x9e;

/// Value Low
/// Value HIGH */ /*rate is 12000000/((1+value)*2)
//pub static DIV_VALUE(rate) = (rate > 6000000)?0:((6000000/rate -1) > 0xffff)? 0xffff: (6000000/rate -1);

/// Commands in MPSSE and Host Emulation Mode
pub const SEND_IMMEDIATE: u8 = 0x87;
pub const WAIT_ON_HIGH: u8 = 0x88;
pub const WAIT_ON_LOW: u8 = 0x89;

/// Commands in Host Emulation Mode
pub const READ_SHORT: u8 = 0x90;
/// Address_Low
pub const READ_EXTENDED: u8 = 0x91;
/// Address High / Address Low
pub const WRITE_SHORT: u8 = 0x92;
/// Address_Low
pub const WRITE_EXTENDED: u8 = 0x93;


/* Definitions for flow control */
/// Reset the port
pub const SIO_RESET: u8 = 0;
/// Set the modem control register
pub const SIO_MODEM_CTRL: u8 = 1;
/// Set flow control register
pub const SIO_SET_FLOW_CTRL: u8 = 2;
/// Set baud rate
pub const SIO_SET_BAUD_RATE: u8 = 3;
/// Set the data characteristics of the port
pub const SIO_SET_DATA: u8 = 4;

pub const FTDI_DEVICE_OUT_REQTYPE: u8 =
    ffi::LIBUSB_REQUEST_TYPE_VENDOR | ffi::LIBUSB_RECIPIENT_DEVICE | ffi::LIBUSB_ENDPOINT_OUT;
pub const FTDI_DEVICE_IN_REQTYPE: u8 =
    ffi::LIBUSB_REQUEST_TYPE_VENDOR | ffi::LIBUSB_RECIPIENT_DEVICE | ffi::LIBUSB_ENDPOINT_IN;

/// Requests
pub const SIO_RESET_REQUEST: u8 = SIO_RESET;
pub const SIO_SET_BAUDRATE_REQUEST: u8 = SIO_SET_BAUD_RATE;
pub const SIO_SET_DATA_REQUEST: u8 = SIO_SET_DATA;
pub const SIO_SET_FLOW_CTRL_REQUEST: u8 = SIO_SET_FLOW_CTRL;
pub const SIO_SET_MODEM_CTRL_REQUEST: u8 = SIO_MODEM_CTRL;
pub const SIO_POLL_MODEM_STATUS_REQUEST: u8 = 0x05;
pub const SIO_SET_EVENT_CHAR_REQUEST: u8 = 0x06;
pub const SIO_SET_ERROR_CHAR_REQUEST: u8 = 0x07;
pub const SIO_SET_LATENCY_TIMER_REQUEST: u8 = 0x09;
pub const SIO_GET_LATENCY_TIMER_REQUEST: u8 = 0x0A;
pub const SIO_SET_BITMODE_REQUEST: u8 = 0x0B;
pub const SIO_READ_PINS_REQUEST: u8 = 0x0C;
pub const SIO_READ_EEPROM_REQUEST: u8 = 0x90;
pub const SIO_WRITE_EEPROM_REQUEST: u8 = 0x91;
pub const SIO_ERASE_EEPROM_REQUEST: u8 = 0x92;

pub const SIO_RESET_SIO: u8 = 0;

/// New names for the values used internally to flush (purge).
pub const SIO_TCIFLUSH: u8 = 2;
pub const SIO_TCOFLUSH: u8 = 1;

pub const SIO_DISABLE_FLOW_CTRL: u8 = 0x0;
pub const SIO_RTS_CTS_HS: u8 = (0x1 << 8) as u8;
pub const SIO_DTR_DSR_HS: u8 = (0x2 << 8) as u8;
pub const SIO_XON_XOFF_HS: u8 = (0x4 << 8) as u8;

pub const SIO_SET_DTR_MASK: u8 = 0x1;
pub const SIO_SET_DTR_HIGH: u8 = (1 | ((SIO_SET_DTR_MASK << 8) as u8) as u8) as u8;
pub const SIO_SET_DTR_LOW: u8 = (0 | ((SIO_SET_DTR_MASK << 8) as u8) as u8) as u8;
pub const SIO_SET_RTS_MASK: u8 = 0x2;
pub const SIO_SET_RTS_HIGH: u8 = (2 | ((SIO_SET_RTS_MASK << 8) as u8) as u8) as u8;
pub const SIO_SET_RTS_LOW: u8 = (0 | ((SIO_SET_RTS_MASK << 8) as u8) as u8) as u8;
