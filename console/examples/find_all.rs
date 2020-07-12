#![allow(unused_imports)]
use ::ftdi_library::ftdi::ftdi_context::ftdi_context;
use ::ftdi_library::ftdi::ftdi_device_list::ftdi_device_list;
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

            match ftdi_device_list::ftdi_usb_find_all(&ftdi_context, 0, 0) {
                Ok(list) => {
                    //print list
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