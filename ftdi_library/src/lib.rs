pub mod ftdi;
pub mod constants_test;

#[cfg(test)]
mod tests {
    use crate::ftdi::ftdi_context::ftdi_context;
    use crate::ftdi::ftdi_device_list::ftdi_device_list;
    use crate::ftdi::ftdi_context::FtdiContextError;
    use snafu::{GenerateBacktrace};

    #[test]
    fn create_new_ftdi_context() {
        let ftdi = ftdi_context::new();
        match ftdi {
            Ok(_) => {/* all is fine */}
            _ => {
                assert!(false); // error
            }
        }
    }

    #[test]
    fn ftdi_usb_find_all() {
        let mut ftdi = ftdi_context::new_with_log_level(None).unwrap();
        let mut ftdi_list = ftdi_device_list::new(&ftdi).unwrap();
        match ftdi_list.ftdi_usb_find_all(&mut ftdi, 0, 0) {
            Ok(_) => {}
            _ => {
                assert!(false); // error
            }
        }
    }

    #[test]
    fn ftdi_usb_open() {
        let mut ftdi = ftdi_context::new_with_log_level(None).unwrap();
        match ftdi.ftdi_usb_open(0, 0) {
            Ok(_) => { /* all is fine */ }
            Err(_) => {
                assert!(false); // error
            }
        }
    }

    #[test]
    fn ftdi_usb_open_desc_index() {
        let mut ftdi = ftdi_context::new().unwrap();
        match ftdi.ftdi_usb_open_desc_index(0, 0, None, None, 0) {
            Ok(_) => { /* all is fine */ }
            Err(_) => {
                assert!(false); // error
            }
        }
    }

    #[test]
    fn parse_vendor_product_index_ok() {
        let values: Vec<(&str, std::result::Result<std::vec::Vec<u16>, FtdiContextError>)> = vec![
            ("s:12:34:0", Ok(vec![12u16, 34u16, 0u16])),
            ("s:0o12:0o74:0o3", Ok(vec![10u16, 60u16, 3u16])),
            ("s:0xAD:0xF34:0x2", Ok(vec![173u16, 3892u16, 2u16])),
            ("s:0xAD:0o34:0", Ok(vec![173u16, 28u16, 0u16])),
            ("i:400:400", Ok(vec![400u16, 400u16])),
            ("i:0x400:0x400", Ok(vec![1024u16, 1024u16])),
            ("i:0o4070:0o4040", Ok(vec![2104u16, 2080u16])),
            ("i:0o4070:0o4040:0o1", Ok(vec![2104u16, 2080u16, 1u16])),
            ("s:400:400:0", Ok(vec![400u16, 400u16, 0u16])),
            ("s:400:0x4DF:0o0", Ok(vec![400u16, 1247u16, 0u16])),
            ("s:0o400:0x4DF:0x0", Ok(vec![256u16, 1247u16, 0u16])),
        ];
        for (input, expected) in values {
            let result = ftdi_context::parse_vendor_product_index(&input);
            assert_eq!(result.unwrap(), expected.unwrap());
        }
    }

    #[test]
    fn parse_vendor_product_index_fail() {
        let values: Vec<(&str, std::result::Result<std::vec::Vec<u16>, FtdiContextError>)> = vec![
            ("", Err(FtdiContextError::UsbCommonError{code: -11,
                message:"incorrect 'description' format or length, see format explanation in code".to_string(), backtrace: GenerateBacktrace::generate()})),
            ("  ", Err(FtdiContextError::UsbCommonError{code: -11,
                message:"incorrect 'description' format or length, see format explanation in code".to_string(), backtrace: GenerateBacktrace::generate()})),
            ("empty", Err(FtdiContextError::UsbCommonError{code: -11,
                message:"incorrect 'description' format or length, see format explanation in code".to_string(), backtrace: GenerateBacktrace::generate()})),
            ("s:", Err(FtdiContextError::UsbCommonError{code: -12,
                message:"incorrect 'description' format, vendor and product is minimal set".to_string(), backtrace: GenerateBacktrace::generate()})),
            (":empty", Err(FtdiContextError::UsbCommonError{code: -12,
                message:"incorrect 'description' format, vendor and product is minimal set".to_string(), backtrace: GenerateBacktrace::generate()})),
            ("s:1234", Err(FtdiContextError::UsbCommonError{code: -12,
                message:"incorrect 'description' format, vendor and product is minimal set".to_string(), backtrace: GenerateBacktrace::generate()})),
            ("s:0o400:0x4DF:0x0:dddd", Err(FtdiContextError::UsbCommonError{code: -14,
                message:"incorrect 'description' format is too long".to_string(), backtrace: GenerateBacktrace::generate()})),
            ("i:0xFFFFF:0x4DF:0x0", Err(FtdiContextError::UsbCommonError{code: -15,
                message:"HEX value parse error".to_string(), backtrace: GenerateBacktrace::generate()})),
            ("i:0xFF:0x4DsdhF:0x0", Err(FtdiContextError::UsbCommonError{code: -15,
                message:"HEX value parse error".to_string(), backtrace: GenerateBacktrace::generate()})),
            ("s:0o8800:0o123:0o0", Err(FtdiContextError::UsbCommonError{code: -16,
                message:"Octal value parse error".to_string(), backtrace: GenerateBacktrace::generate()})),
            ("s:0o56:0o123678:0o0", Err(FtdiContextError::UsbCommonError{code: -16,
                message:"Octal value parse error".to_string(), backtrace: GenerateBacktrace::generate()})),
            ("i:657777:0x4DF:0x0", Err(FtdiContextError::UsbCommonError{code: -17,
                message:"Decimal value parse error".to_string(), backtrace: GenerateBacktrace::generate()})),
            ("i:657777:0x4DF:0x0", Err(FtdiContextError::UsbCommonError{code: -17,
                message:"Decimal value parse error".to_string(), backtrace: GenerateBacktrace::generate()})),
            ("i:657:67000:0", Err(FtdiContextError::UsbCommonError{code: -17,
                message:"Decimal value parse error".to_string(), backtrace: GenerateBacktrace::generate()})),
            ("i:657:600:0789056", Err(FtdiContextError::UsbCommonError{code: -17,
                message:"Decimal value parse error".to_string(), backtrace: GenerateBacktrace::generate()})),
            ("s:124:", Err(FtdiContextError::UsbCommonError{code: -17,
                message:"Decimal value parse error".to_string(), backtrace: GenerateBacktrace::generate()})),
        ];
        for (input, expected) in values {
            println!("input = \'{}\'", input);
            let result = ftdi_context::parse_vendor_product_index(&input);
            assert_eq!(result.err().unwrap(), expected.err().unwrap());
        }
    }

    #[test]
    fn parse_number_str() {
        let values: Vec<(&str, Option<u16>)> = vec![
            ("", None),
            ("  ", None),
            ("empty", None),
            ("s", None),
            ("1234", Some(1234)),
            ("12", Some(12)),
            ("0o1274", Some(700)),
            ("0xADF", Some(2783)),
        ];
        for (input, expected) in values {
            println!("input = \'{}\'", input);
            let result = ftdi_context::parse_number_str(&input);
            assert_eq!(result, expected);
        }
    }

}
