pub mod ftdi;
pub mod constants_test;

#[cfg(test)]
mod tests {
    use crate::ftdi::ftdi_context::ftdi_context;
    use crate::ftdi::ftdi_device_list::ftdi_device_list;

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
        let values = vec![
            ("s:12:34:0", vec![12u16, 34u16, 0u16]),
            ("i:400:400", vec![400u16, 400u16]),
            ("i:400:400:0", vec![400u16, 400u16, 0u16]),
            ("s:400:400:0", vec![400u16, 400u16, 0u16]),
        ];
        for (mut input, expected) in values {
            let result = ftdi_context::parse_vendor_product_index(&input);
            assert_eq!(result, expected);
        }
    }

    #[test]
    #[should_panic(expected = "")]
    fn parse_vendor_product_index_fail() {

    }

}
