#![allow(non_camel_case_types)]
#![allow(dead_code)]

pub const FTDI_MAX_EEPROM_SIZE: usize = 256;

pub const MAX_POWER_MILLIAMP_PER_UNIT: u8 = 2;

// #[derive(Copy, Clone, Debug)]
// #[derive(Debug)]
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
    pub use_usb_version: i32,
    /// maximum power
    pub max_power: i32,

    /// manufacturer name
    pub manufacturer: [u8; 256],
    // pub manufacturer: String,
    /// product name
    // pub product: String,
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
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum ftdi_eeprom_value {
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
impl From<u8> for ftdi_eeprom_value {
    fn from(value: u8) -> ftdi_eeprom_value {
        match value {
            0 => ftdi_eeprom_value::VENDOR_ID,
            1 => ftdi_eeprom_value::PRODUCT_ID,
            2 => ftdi_eeprom_value::SELF_POWERED,
            3 => ftdi_eeprom_value::REMOTE_WAKEUP,
            4 => ftdi_eeprom_value::IS_NOT_PNP,
            5 => ftdi_eeprom_value::SUSPEND_DBUS7,
            6 => ftdi_eeprom_value::IN_IS_ISOCHRONOUS,
            7 => ftdi_eeprom_value::OUT_IS_ISOCHRONOUS,
            8 => ftdi_eeprom_value::SUSPEND_PULL_DOWNS,
            9 => ftdi_eeprom_value::USE_SERIAL,
            10 => ftdi_eeprom_value::USB_VERSION,
            11 => ftdi_eeprom_value::USE_USB_VERSION,
            12 => ftdi_eeprom_value::MAX_POWER,
            13 => ftdi_eeprom_value::CHANNEL_A_TYPE,
            14 => ftdi_eeprom_value::CHANNEL_B_TYPE,
            15 => ftdi_eeprom_value::CHANNEL_A_DRIVER,
            16 => ftdi_eeprom_value::CHANNEL_B_DRIVER,
            17 => ftdi_eeprom_value::CBUS_FUNCTION_0,
            18 => ftdi_eeprom_value::CBUS_FUNCTION_1,
            19 => ftdi_eeprom_value::CBUS_FUNCTION_2,
            20 => ftdi_eeprom_value::CBUS_FUNCTION_3,
            21 => ftdi_eeprom_value::CBUS_FUNCTION_4,
            22 => ftdi_eeprom_value::CBUS_FUNCTION_5,
            23 => ftdi_eeprom_value::CBUS_FUNCTION_6,
            24 => ftdi_eeprom_value::CBUS_FUNCTION_7,
            25 => ftdi_eeprom_value::CBUS_FUNCTION_8,
            26 => ftdi_eeprom_value::CBUS_FUNCTION_9,
            27 => ftdi_eeprom_value::HIGH_CURRENT,
            28 => ftdi_eeprom_value::HIGH_CURRENT_A,
            29 => ftdi_eeprom_value::HIGH_CURRENT_B,
            30 => ftdi_eeprom_value::INVERT,
            31 => ftdi_eeprom_value::GROUP0_DRIVE,
            32 => ftdi_eeprom_value::GROUP0_SCHMITT,
            33 => ftdi_eeprom_value::GROUP0_SLEW,
            34 => ftdi_eeprom_value::GROUP1_DRIVE,
            35 => ftdi_eeprom_value::GROUP1_SCHMITT,
            36 => ftdi_eeprom_value::GROUP1_SLEW,
            37 => ftdi_eeprom_value::GROUP2_DRIVE,
            38 => ftdi_eeprom_value::GROUP2_SCHMITT,
            39 => ftdi_eeprom_value::GROUP2_SLEW,
            40 => ftdi_eeprom_value::GROUP3_DRIVE,
            41 => ftdi_eeprom_value::GROUP3_SCHMITT,
            42 => ftdi_eeprom_value::GROUP3_SLEW,
            43 => ftdi_eeprom_value::CHIP_SIZE,
            44 => ftdi_eeprom_value::CHIP_TYPE,
            45 => ftdi_eeprom_value::POWER_SAVE,
            46 => ftdi_eeprom_value::CLOCK_POLARITY,
            47 => ftdi_eeprom_value::DATA_ORDER,
            48 => ftdi_eeprom_value::FLOW_CONTROL,
            49 => ftdi_eeprom_value::CHANNEL_C_DRIVER,
            50 => ftdi_eeprom_value::CHANNEL_D_DRIVER,
            51 => ftdi_eeprom_value::CHANNEL_A_RS485,
            52 => ftdi_eeprom_value::CHANNEL_B_RS485,
            53 => ftdi_eeprom_value::CHANNEL_C_RS485,
            54 => ftdi_eeprom_value::CHANNEL_D_RS485,
            55 => ftdi_eeprom_value::RELEASE_NUMBER,
            56 => ftdi_eeprom_value::EXTERNAL_OSCILLATOR,
            57 => ftdi_eeprom_value::USER_DATA_ADDR,
            _ => panic!("ftdi_eeprom_value is unknown for value = {}", value),
        }
    }
}