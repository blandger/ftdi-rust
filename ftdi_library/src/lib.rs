pub mod ftdi;
pub mod constants_test;

#[cfg(test)]
mod tests {
    use crate::ftdi::ftdi_context::ftdi_context;
    use crate::ftdi::ftdi_device_list::ftdi_device_list;
    use crate::ftdi::core::FtdiError;

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
        let ftdi = ftdi_context::new().unwrap();
        let mut ftdi_list = ftdi_device_list::new(&ftdi).unwrap();
        match ftdi_list.ftdi_usb_find_all(&ftdi, 0, 0) {
            Ok(_) => {}
            _ => {
                assert!(false); // error
            }
        }
    }

    #[test]
    fn ftdi_usb_open() {
        let mut ftdi = ftdi_context::new().unwrap();
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
        let values: Vec<(&str, std::result::Result<std::vec::Vec<u16>, FtdiError>)> = vec![
            ("s:12:34:0", Ok(vec![12u16, 34u16, 0u16])),
            ("s:0o12:0o74:0o3", Ok(vec![10u16, 60u16, 3u16])),
            ("s:0xAD:0xF34:0x2", Ok(vec![173u16, 3892u16, 2u16])),
            ("s:0xAD:0o34:0", Ok(vec![173u16, 28u16, 0u16])),
            ("i:400:400", Ok(vec![400u16, 400u16])),
            ("i:0x400:0x400", Ok(vec![1024u16, 1024u16])),
            ("i:0o4070:0o4040:0o1", Ok(vec![2104u16, 2080u16, 1u16])),
            ("s:400:400:0", Ok(vec![400u16, 400u16, 0u16])),
            ("s:400:0x4DF:0o0", Ok(vec![400u16, 1247u16, 0u16])),
            ("s:0o400:0x4DF:0x0", Ok(vec![256u16, 1247u16, 0u16])),
        ];
        for (mut input, expected) in values {
            let result = ftdi_context::parse_vendor_product_index(&input);
            assert_eq!(result.unwrap(), expected.unwrap());
        }
    }

    #[test]
    #[should_panic(expected = "")]
    fn parse_vendor_product_index_fail() {

    }

}
