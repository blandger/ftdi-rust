pub mod ftdi;

#[cfg(test)]
mod tests {
    use crate::ftdi::constants::{
        ftdi_chip_type, ftdi_parity_type, ftdi_stopbits_type, ftdi_bits_type,
        ftdi_break_type, ftdi_mpsse_mode, ftdi_interface, ftdi_module_detach_mode
    };
    use crate::ftdi::eeprom::{ftdi_eeprom_value};
    use crate::ftdi::core::ftdi_context;

    #[test]
    fn ftdi_chip_type_conversion() {
        assert_eq!(ftdi_chip_type::TYPE_AM, ftdi_chip_type::from(0 as u8));
        assert_eq!(ftdi_chip_type::TYPE_BM, ftdi_chip_type::from(1 as u8));
        assert_eq!(ftdi_chip_type::TYPE_2232C, ftdi_chip_type::from(2 as u8));
        assert_eq!(ftdi_chip_type::TYPE_R, ftdi_chip_type::from(3 as u8));
        assert_eq!(ftdi_chip_type::TYPE_2232H, ftdi_chip_type::from(4 as u8));
        assert_eq!(ftdi_chip_type::TYPE_4232H, ftdi_chip_type::from(5 as u8));
        assert_eq!(ftdi_chip_type::TYPE_232H, ftdi_chip_type::from(6 as u8));
        assert_eq!(ftdi_chip_type::TYPE_230X, ftdi_chip_type::from(7 as u8));
    }
    #[test]
    #[should_panic(expected = "ftdi_chip_type is unknown for value = 8")]
    fn ftdi_chip_type_conversion_fail() {
        ftdi_chip_type::from(8 as u8);
    }

    #[test]
    fn ftdi_parity_type_conversion() {
        assert_eq!(ftdi_parity_type::NONE, ftdi_parity_type::from(0 as u8));
        assert_eq!(ftdi_parity_type::ODD, ftdi_parity_type::from(1 as u8));
        assert_eq!(ftdi_parity_type::EVEN, ftdi_parity_type::from(2 as u8));
        assert_eq!(ftdi_parity_type::MARK, ftdi_parity_type::from(3 as u8));
        assert_eq!(ftdi_parity_type::SPACE, ftdi_parity_type::from(4 as u8));
    }
    #[test]
    #[should_panic(expected = "ftdi_parity_type is unknown for value = 8")]
    fn ftdi_parity_type_conversion_fail() {
        ftdi_parity_type::from(8 as u8);
    }

    #[test]
    fn ftdi_stopbits_type_conversion() {
        assert_eq!(ftdi_stopbits_type::STOP_BIT_1, ftdi_stopbits_type::from(0 as u8));
        assert_eq!(ftdi_stopbits_type::STOP_BIT_15, ftdi_stopbits_type::from(1 as u8));
        assert_eq!(ftdi_stopbits_type::STOP_BIT_2, ftdi_stopbits_type::from(2 as u8));
    }
    #[test]
    #[should_panic(expected = "ftdi_stopbits_type is unknown for value = 8")]
    fn ftdi_stopbits_type_conversion_fail() {
        ftdi_stopbits_type::from(8 as u8);
    }

    #[test]
    fn ftdi_bits_type_conversion() {
        assert_eq!(ftdi_bits_type::BITS_7, ftdi_bits_type::from(7 as u8));
        assert_eq!(ftdi_bits_type::BITS_8, ftdi_bits_type::from(8 as u8));
    }
    #[test]
    #[should_panic(expected = "ftdi_bits_type is unknown for value = 0")]
    fn ftdi_bits_type_conversion_fail() {
        ftdi_bits_type::from(0 as u8);
    }

    #[test]
    fn ftdi_break_type_conversion() {
        assert_eq!(ftdi_break_type::BREAK_OFF, ftdi_break_type::from(0 as u8));
        assert_eq!(ftdi_break_type::BREAK_ON, ftdi_break_type::from(1 as u8));
    }
    #[test]
    #[should_panic(expected = "ftdi_break_type is unknown for value = 12")]
    fn ftdi_break_type_conversion_fail() {
        ftdi_break_type::from(12 as u8);
    }

    #[test]
    fn ftdi_mpsse_mode_conversion() {
        assert_eq!(ftdi_mpsse_mode::BITMODE_RESET,  ftdi_mpsse_mode::from(0 as u8));
        assert_eq!(ftdi_mpsse_mode::BITMODE_BITBANG, ftdi_mpsse_mode::from(1 as u8));
        assert_eq!(ftdi_mpsse_mode::BITMODE_MPSSE, ftdi_mpsse_mode::from(2 as u8));
        assert_eq!(ftdi_mpsse_mode::BITMODE_SYNCBB, ftdi_mpsse_mode::from(4 as u8));
        assert_eq!(ftdi_mpsse_mode::BITMODE_MCU, ftdi_mpsse_mode::from(8 as u8));
        assert_eq!(ftdi_mpsse_mode::BITMODE_OPTO, ftdi_mpsse_mode::from(10 as u8));
        assert_eq!(ftdi_mpsse_mode::BITMODE_CBUS, ftdi_mpsse_mode::from(20 as u8));
        assert_eq!(ftdi_mpsse_mode::BITMODE_SYNCFF, ftdi_mpsse_mode::from(40 as u8));
        assert_eq!(ftdi_mpsse_mode::BITMODE_FT1284, ftdi_mpsse_mode::from(80 as u8));
    }
    #[test]
    #[should_panic(expected = "ftdi_mpsse_mode is unknown for value = 12")]
    fn ftdi_mpsse_mode_conversion_fail() {
        ftdi_mpsse_mode::from(12 as u8);
    }

    #[test]
    fn ftdi_interface_conversion() {
        assert_eq!(ftdi_interface::INTERFACE_ANY,  ftdi_interface::from(0 as u8));
        assert_eq!(ftdi_interface::INTERFACE_A, ftdi_interface::from(1 as u8));
        assert_eq!(ftdi_interface::INTERFACE_B, ftdi_interface::from(2 as u8));
        assert_eq!(ftdi_interface::INTERFACE_C, ftdi_interface::from(3 as u8));
        assert_eq!(ftdi_interface::INTERFACE_D, ftdi_interface::from(4 as u8));
    }
    #[test]
    #[should_panic(expected = "ftdi_interface is unknown for value = 12")]
    fn ftdi_interface_conversion_fail() {
        ftdi_interface::from(12 as u8);
    }

    #[test]
    fn ftdi_module_detach_mode_conversion() {
        assert_eq!(ftdi_module_detach_mode::AUTO_DETACH_SIO_MODULE,  ftdi_module_detach_mode::from(0 as u8));
        assert_eq!(ftdi_module_detach_mode::DONT_DETACH_SIO_MODULE, ftdi_module_detach_mode::from(1 as u8));
        assert_eq!(ftdi_module_detach_mode::AUTO_DETACH_REATACH_SIO_MODULE, ftdi_module_detach_mode::from(2 as u8));
    }
    #[test]
    #[should_panic(expected = "ftdi_module_detach_mode is unknown for value = 12")]
    fn ftdi_module_detach_mode_conversion_fail() {
        ftdi_module_detach_mode::from(12 as u8);
    }

    #[test]
    fn fftdi_eeprom_value_conversion() {
        assert_eq!(ftdi_eeprom_value::VENDOR_ID,  ftdi_eeprom_value::from(0 as u8));
        assert_eq!(ftdi_eeprom_value::PRODUCT_ID, ftdi_eeprom_value::from(1 as u8));
        assert_eq!(ftdi_eeprom_value::HIGH_CURRENT_A, ftdi_eeprom_value::from(28 as u8));
        assert_eq!(ftdi_eeprom_value::USER_DATA_ADDR, ftdi_eeprom_value::from(57 as u8));
    }
    #[test]
    #[should_panic(expected = "ftdi_eeprom_value is unknown for value = 200")]
    fn ftdi_eeprom_value_conversion_fail() {
        ftdi_eeprom_value::from(200 as u8);
    }

    #[test]
    fn create_new_ftdi_context() {
        let created_ftdi_context_result = ftdi_context::new();
        match created_ftdi_context_result {
            Ok(_) => {/* all is fine */}
            _ => {
                assert!(false); // error
            }
        }
    }
}
