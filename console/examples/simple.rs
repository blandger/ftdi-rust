use ::ftdi_library::ftdi::core::*;
use ::ftdi_library::ftdi::ftdi_context::ftdi_context;
use ::ftdi_library::ftdi::ftdi_version_info::ftdi_version_info;
use log::{info};
use log4rs;

fn main() -> Result<(), FtdiError> {
    // log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    match log4rs::init_file("log4rs.yaml", Default::default()) {
        Ok(result) => println!("OK with log config = {:?}", result),
        Err(error) => println!("Log config not found, {}", error),
    }

    info!("booting up...");
    let mut ftdi_context = ftdi_context::new()?;
    info!("ftdi context in created - OK");

    let version = ftdi_version_info::ftdi_get_library_version();
    info!("Initialized libftdi {} (major: {}, minor: {}, micro: {}, snapshot ver: {})\n",
          version.version_str, version.major, version.minor, version.micro,
          version.snapshot_str);

    // ftdi_context.set_interface_type(ftdi_interface::INTERFACE_ANY);

    // ftdi_context.ftdi_usb_open(0x0403, 0x6001)?; // fetch ony FTDI devices
    ftdi_context.ftdi_usb_open_desc_index(0, 0, None, None, 0)?; // fetch all devices
    Ok(())
}
