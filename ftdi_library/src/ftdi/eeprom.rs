#![allow(non_camel_case_types)]
#![allow(dead_code)]

pub const FTDI_MAX_EEPROM_SIZE: usize = 256;

pub const MAX_POWER_MILLIAMP_PER_UNIT: u8 = 2;

/// brief FTDI eeprom structure
pub struct ftdi_eeprom {
    /// vendor id
    pub vendor_id: i32,
    /// product id
    pub product_id: i32,

    /// Was the eeprom structure initialized for the actual connected device?
    pub initialized_for_connected_device: bool,

    /// self powered
    pub self_powered: i32,
    /// remote wakeup
    pub remote_wakeup: i32,

    pub is_not_pnp: bool,

    /// Suspend on DBUS7 Low
    pub suspend_dbus7: i32,

    /// input in isochronous transfer mode
    pub in_is_isochronous: bool,
    /// output in isochronous transfer mode
    pub out_is_isochronous: bool,
    /// suspend pull downs
    pub suspend_pull_downs: i32,

    /// use serial
    pub use_serial: bool,
    /// usb version
    pub usb_version: i32,
    // Use usb version on FT2232 devices
    use_usb_version: i32,
    /// maximum power
    pub max_power: i32,

    /// manufacturer name
    pub manufacturer: [u8; 256],
    /// product name
    pub product: [u8; 256],
    /// serial number
    pub serial: [u8; 256],

    /// 2232D/H specific
    /// Hardware type, 0 = RS232 Uart, 1 = 245 FIFO, 2 = CPU FIFO, 4 = OPTO Isolate
    pub channel_a_type: i32,
    pub channel_b_type: i32,
    /// Driver Type, 1 = VCP
    pub channel_a_driver: i32,
    pub channel_b_driver: i32,
    pub channel_c_driver: i32,
    pub channel_d_driver: i32,
    /// 4232H specific
    pub channel_a_rs485enable: bool,
    pub channel_b_rs485enable: bool,
    pub channel_c_rs485enable: bool,
    pub channel_d_rs485enable: bool,

    /// Special function of FT232R/FT232H devices (and possibly others as well)
    /// CBUS pin function. See CBUS_xxx defines.
    pub cbus_function: [i32;10],
    /// Select high current drive on R devices.
    pub high_current: i32,
    /// Select high current drive on A channel (2232C).
    pub high_current_a: i32,
    /// Select high current drive on B channel (2232C).
    pub high_current_b: i32,
    /// Select inversion of data lines (bitmask).
    pub invert: i32,
    /// Enable external oscillator.
    pub external_oscillator: i32,

    /// 2232H/4432H Group specific values
    /// Group0 is AL on 2322H and A on 4232H
    /// Group1 is AH on 2232H and B on 4232H
    /// Group2 is BL on 2322H and C on 4232H
    /// Group3 is BH on 2232H and C on 4232H*/
    pub group0_drive: i32,
    pub group0_schmitt: i32,
    pub group0_slew: i32,
    pub group1_drive: i32,
    pub group1_schmitt: i32,
    pub group1_slew: i32,
    pub group2_drive: i32,
    pub group2_schmitt: i32,
    pub group2_slew: i32,
    pub group3_drive: i32,
    pub group3_schmitt: i32,
    pub group3_slew: i32,

    pub powersave: i32,

    pub clock_polarity: i32,
    pub data_order: i32,
    pub flow_control: i32,

    /// user data
    pub user_data_addr: i32,
    pub user_data_size: i32,
    pub user_data: [u8; 256],

    /// eeprom size in bytes. This doesn't get stored in the eeprom but is the only way to pass it to ftdi_eeprom_build.
    pub size: i32,
    /// EEPROM Type 0x46 for 93xx46, 0x56 for 93xx56 and 0x66 for 93xx66
    pub chip: i32,
    pub buf: [u8; FTDI_MAX_EEPROM_SIZE],

    /// device release number
    pub release_number: i32,
}

/// List all handled EEPROM values.
// Append future new values only at the end to provide API/ABI stability
enum ftdi_eeprom_value {
    VENDOR_ID          = 0,
    PRODUCT_ID         = 1,
    SELF_POWERED       = 2,
    REMOTE_WAKEUP      = 3,
    IS_NOT_PNP         = 4,
    SUSPEND_DBUS7      = 5,
    IN_IS_ISOCHRONOUS  = 6,
    OUT_IS_ISOCHRONOUS = 7,
    SUSPEND_PULL_DOWNS = 8,
    USE_SERIAL         = 9,
    USB_VERSION        = 10,
    USE_USB_VERSION    = 11,
    MAX_POWER          = 12,
    CHANNEL_A_TYPE     = 13,
    CHANNEL_B_TYPE     = 14,
    CHANNEL_A_DRIVER   = 15,
    CHANNEL_B_DRIVER   = 16,
    CBUS_FUNCTION_0    = 17,
    CBUS_FUNCTION_1    = 18,
    CBUS_FUNCTION_2    = 19,
    CBUS_FUNCTION_3    = 20,
    CBUS_FUNCTION_4    = 21,
    CBUS_FUNCTION_5    = 22,
    CBUS_FUNCTION_6    = 23,
    CBUS_FUNCTION_7    = 24,
    CBUS_FUNCTION_8    = 25,
    CBUS_FUNCTION_9    = 26,
    HIGH_CURRENT       = 27,
    HIGH_CURRENT_A     = 28,
    HIGH_CURRENT_B     = 29,
    INVERT             = 30,
    GROUP0_DRIVE       = 31,
    GROUP0_SCHMITT     = 32,
    GROUP0_SLEW        = 33,
    GROUP1_DRIVE       = 34,
    GROUP1_SCHMITT     = 35,
    GROUP1_SLEW        = 36,
    GROUP2_DRIVE       = 37,
    GROUP2_SCHMITT     = 38,
    GROUP2_SLEW        = 39,
    GROUP3_DRIVE       = 40,
    GROUP3_SCHMITT     = 41,
    GROUP3_SLEW        = 42,
    CHIP_SIZE          = 43,
    CHIP_TYPE          = 44,
    POWER_SAVE         = 45,
    CLOCK_POLARITY     = 46,
    DATA_ORDER         = 47,
    FLOW_CONTROL       = 48,
    CHANNEL_C_DRIVER   = 49,
    CHANNEL_D_DRIVER   = 50,
    CHANNEL_A_RS485    = 51,
    CHANNEL_B_RS485    = 52,
    CHANNEL_C_RS485    = 53,
    CHANNEL_D_RS485    = 54,
    RELEASE_NUMBER     = 55,
    EXTERNAL_OSCILLATOR= 56,
    USER_DATA_ADDR     = 57,
}
