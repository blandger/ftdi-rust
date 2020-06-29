#![allow(unused_imports)]
use ::ftdi_library::ftdi::core::*;
use log::info;
use log4rs;

fn main() {
    log4rs::init_file("./log4rs.yaml", Default::default()).unwrap();
    info!("booting up");
    let _ftdi_context = ftdi_context::new();

}