#![allow(unused_imports)]
use ::ftdi_library::ftdi::core::{FtdiError};
use ::ftdi_library::ftdi::ftdi_context::ftdi_context;
use ::ftdi_library::ftdi::ftdi_device_list::ftdi_device_list;
use log::{info, error};
use log4rs;

fn main() -> Result<(), FtdiError> {
    match log4rs::init_file("log4rs.yaml", Default::default()) {
        Ok(result) => println!("OK with log config = {:?}", result),
        Err(error) => println!("Log config not found, {}", error),
    }
    info!("booting up...");
    let ftdi_context = ftdi_context::new()?;
    info!("ftdi context in created - OK");

    info!("start find all usb device(s)...");
    let list = ftdi_device_list::ftdi_usb_find_all(&ftdi_context, 0, 0)?;
    info!("Number of FTDI devices found: {} - OK", list.number_found_devices);
    for (index, _device) in list.system_device_list.iter().enumerate() {
        info!("Checking device: {}", index);
        // let (manufacturer, description) = ftdi_usb_get_strings(&ftdi_context_result, &device);
    }
    Ok(())
}