#![allow(unused_imports)]
use ::ftdi_library::ftdi::core::*;
use ::ftdi_library::ftdi::constants::ftdi_interface;
use ::ftdi_library::ftdi::ftdi_context::ftdi_context;
use ::ftdi_library::ftdi::ftdi_device_list::ftdi_device_list;
use ::ftdi_library::ftdi::ftdi_version_info::ftdi_version_info;
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

/*            // match ftdi_context.ftdi_usb_open_desc_index(0x0403, 0x6001, None, None, 0) {
            match ftdi_context.ftdi_usb_open_desc_index(0, 0, None, None, 0) {
                Ok(ftdi_context) => {
                    info!("ftdi device list is {} OK !", ftdi_context.);
                }
                Err(error) => {
                    error!("There is get Usb Device List {}", error);
                }
            }
*/
        }
        Err(err) => {
            error!("There is {}", err)
        }
    }

    let version = ftdi_version_info::ftdi_get_library_version();
    info!("Initialized libftdi {} (major: {}, minor: {}, micro: {}, snapshot ver: {})\n",
           version.version_str, version.major, version.minor, version.micro,
           version.snapshot_str);
}
