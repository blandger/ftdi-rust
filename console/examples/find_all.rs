use ::ftdi_library::ftdi::ftdi_context::ftdi_context;
use ::ftdi_library::ftdi::ftdi_device_list::ftdi_device_list;
use log::{info};
use log4rs;
use ftdi_library::ftdi::ftdi_context::FtdiContextError;

#[cfg(target_os = "linux")]
const PATH_TO_YAML_LOG_CONFIG:&'static str = "./log4rs.yaml"; // string path to log config
#[cfg(any(target_os = "windows", target_os = "macos"))]
const PATH_TO_YAML_LOG_CONFIG:&'static str = "log4rs.yaml";

fn main() -> Result<(), FtdiContextError> {
    match log4rs::init_file(PATH_TO_YAML_LOG_CONFIG, Default::default()) {
        Ok(_) => println!("log4rs config file is found - OK"),
        Err(error) => println!("Log config not found as \'{}\', error: \'{}\'", PATH_TO_YAML_LOG_CONFIG, error),
    }
    info!("booting up...");
    let mut ftdi = ftdi_context::new()?;
    info!("ftdi context in created - OK");

    info!("start find all usb device(s)...");
    let mut ftdi_list = ftdi_device_list::new(&ftdi)?;
    let list = ftdi_list.ftdi_usb_find_all(&mut ftdi,0, 0)?;
    info!("Number of FTDI devices found: [{}] - OK", list.number_found_devices);
    info!("List of FTDI usb devices found: \'{:?}\' - OK", list.system_device_list);
    for (index, device) in list.system_device_list.iter().enumerate() {
        info!("Checking device: {}", index);
        let manufacturer_description = ftdi.ftdi_usb_get_strings(*device)?;
        info!("FTDI chip Manufacturer: {:?}, Description: {:?}, Serial: {:?}\n\n",
              manufacturer_description.0, manufacturer_description.1, manufacturer_description.2);
    }
    Ok(())
}