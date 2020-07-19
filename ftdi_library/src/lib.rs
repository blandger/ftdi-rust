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
        let ftdi = ftdi_context::new();
        match ftdi_device_list::ftdi_usb_find_all(&ftdi.unwrap(), 0, 0) {
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

}
