use ::ftdi_library::ftdi::core::{FtdiError};
use ::ftdi_library::ftdi::ftdi_context::ftdi_context;
use ::ftdi_library::ftdi::ftdi_device_list::ftdi_device_list;
use log::{info};
use log4rs;

fn main() -> Result<(), FtdiError> {
    match log4rs::init_file("log4rs.yaml", Default::default()) {
        Ok(result) => println!("OK with log config = {:?}", result),
        Err(error) => println!("Log config not found, {}", error),
    }
    info!("booting up...");
    let mut ftdi = ftdi_context::new()?;
    info!("ftdi context in created - OK");

    info!("start find all usb device(s)...");
    let list = ftdi_device_list::ftdi_usb_find_all(&ftdi, 0, 0)?;
    info!("Number of FTDI devices found: {} - OK", list.number_found_devices);
    for (index, device) in list.system_device_list.iter().enumerate() {
        info!("Checking device: {}", index);
        let manufacturer_description = ftdi.ftdi_usb_get_strings(*device)?;
        info!("FTDI chip Manufacturer: {:?}, Description: {:?}, Serial: {:?}\n\n",
              manufacturer_description.0, manufacturer_description.1, manufacturer_description.2);
    }
    Ok(())
}