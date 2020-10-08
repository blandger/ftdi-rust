use ::ftdi_library::ftdi::core::*;
use ::ftdi_library::ftdi::ftdi_context::ftdi_context;
use ::ftdi_library::ftdi::ftdi_version_info::ftdi_version_info;
use log::{info};
use log4rs;
use ftdi_library::ftdi::constants::ftdi_chip_type;
use ftdi_library::ftdi::ftdi_context::FtdiContextError;
use libc::{c_int};

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
    let mut ftdi = ftdi_context::new(Some(4))?; // ffi::LIBUSB_LOG_LEVEL_DEBUG
    info!("ftdi context in created - OK");

    let version = ftdi_version_info::ftdi_get_library_version();
    info!("Initialized libftdi {} (major: {}, minor: {}, micro: {}, snapshot ver: {})\n",
          version.version_str, version.major, version.minor, version.micro,
          version.snapshot_str);

    ftdi.ftdi_usb_open(0x0403, 0x6011)?; // 0x6001... fetch ony FTDI devices
    // ftdi.ftdi_usb_open(0, 0)?; // fetch all devices
    // ftdi.ftdi_usb_open_desc_index(0, 0, None, None, 0)?; // fetch all devices

    if ftdi.r#type == ftdi_chip_type::TYPE_R {
        let chipid= ftdi.ftdi_read_chipid()?;
        println!("FTDI chipid = {}", chipid);
        info!("FTDI chipid = {}", chipid);
    }
    Ok(())
}
