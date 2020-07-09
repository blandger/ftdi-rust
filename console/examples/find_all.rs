#![allow(unused_imports)]
use ::ftdi_library::ftdi::core::{ftdi_context, ftdi_device_list};
use ::ftdi_library::ftdi::constants::ftdi_interface;
use log::{info, error};
use log4rs;

fn main() {
    log4rs::init_file("./log4rs.yaml", Default::default()).unwrap();
    info!("booting up");
    let created_ftdi_context_result = ftdi_context::new();
    match created_ftdi_context_result {
        Ok(mut ftdi_context) => {
            info!("ftdi context in created OK");
            ftdi_context.set_interface_type(ftdi_interface::INTERFACE_ANY);

            // match ftdi_context.ftdi_usb_open_desc_index(0x0403, 0x6001, None, None, 0) {
            match ftdi_context.ftdi_usb_open_desc_index(0, 0, None, None, 0) {
                Ok(list) => {
                    info!("ftdi device list is {} OK !", list.ftdi_device_list.len());
                }
                Err(error) => {
                    error!("There is get Usb Device List {}", error);
                }
            }
        }
        Err(err) => {
            error!("There is Init {}", err)
        }
    }

}