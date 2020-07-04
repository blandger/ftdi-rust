#![allow(unused_imports)]
use ::ftdi_library::ftdi::core::*;
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
        }
        Err(err) => {
            error!("There is {}", err)
        }
    }

}