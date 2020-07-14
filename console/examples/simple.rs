use ::ftdi_library::ftdi::core::*;
use ::ftdi_library::ftdi::ftdi_context::ftdi_context;
use ::ftdi_library::ftdi::ftdi_version_info::ftdi_version_info;
use log::{info};
use log4rs;
use ftdi_library::ftdi::constants::ftdi_chip_type;

fn main() -> Result<(), FtdiError> {
    // log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    match log4rs::init_file("log4rs.yaml", Default::default()) {
        Ok(result) => println!("OK with log config = {:?}", result),
        Err(error) => println!("Log config not found, {}", error),
    }

    info!("booting up...");
    let mut ftdi = ftdi_context::new()?;
    info!("ftdi context in created - OK");

    let version = ftdi_version_info::ftdi_get_library_version();
    info!("Initialized libftdi {} (major: {}, minor: {}, micro: {}, snapshot ver: {})\n",
          version.version_str, version.major, version.minor, version.micro,
          version.snapshot_str);

    ftdi.ftdi_usb_open(0x0403, 0x6001)?; // fetch ony FTDI devices
    // ftdi.ftdi_usb_open_desc_index(0, 0, None, None, 0)?; // fetch all devices

    if ftdi.r#type == ftdi_chip_type::TYPE_R {
        let chipid= ftdi.ftdi_read_chipid()?;
        println!("FTDI chipid = {}", chipid);
        info!("FTDI chipid = {}", chipid);
    }
    Ok(())
}
