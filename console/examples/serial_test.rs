use ::ftdi_library::ftdi::ftdi_context::ftdi_context;
use ::ftdi_library::ftdi::ftdi_device_list::ftdi_device_list;
use log::{debug, info, error};
use log4rs;
use signal_hook;
use std::sync::Arc;
use std::{
    {thread, time},
    sync::{
        atomic,
        atomic::{AtomicBool, Ordering}
    }
};
use clap::{value_t, Arg, App};
use ftdi_library::ftdi::constants::{ftdi_interface, ftdi_stopbits_type, ftdi_bits_type, ftdi_parity_type};
use ftdi_library::ftdi::core::FtdiError;
use snafu::{GenerateBacktrace};

#[cfg(target_os = "linux")]
const PATH_TO_YAML_LOG_CONFIG:&'static str = "log4rs.yaml"; // string path to log config
#[cfg(any(target_os = "windows", target_os = "macos"))]
const PATH_TO_YAML_LOG_CONFIG:&'static str = "log4rs.yaml";

#[cfg(not(windows))] // not for windows !
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // construction for command line parameters
    let matches = App::new("Simple serial test check read/write.")
        .version("v 0.1")
        .author("Blandger <blandger@gmail.com>")
        .about("FTDI serial read/write test")
        .arg(Arg::with_name("interface")
            .short("i")
            .long("interface")
            .value_name("INTERFACE")
            .help("INTERFACE_ANY | A | B | C | D, values: 0 - 4")
            .default_value("0")
        )
        .arg(Arg::with_name("v")
            .short("v")
            .long("vendorId")
            .value_name("Vendor ID")
            .help("Vendor ID usb value, default is '0403' for FTDI")
            .default_value("0x0403"))
        .arg(Arg::with_name("p")
            .short("p")
            .long("productId")
            .value_name("Product ID")
            .help("Product ID usb value, usual FTDI values are :0x6001, 0x6010, 0x6011, 0x6014, 0x6015")
            .required(true))
        .arg(Arg::with_name("b")
            .short("b")
            .long("baudrate")
            .value_name("Baud rate / speed")
            .help("Baudrate usb value, default is '115200'")
            .default_value("115200"))
        .arg(Arg::with_name("w")
            .short("w")
            .long("pattern")
            .value_name("pattern one byte value to write")
            .help("Write a pattern as one byte value"))
        .get_matches();
    // try to load yaml logging config file
    match log4rs::init_file(PATH_TO_YAML_LOG_CONFIG, Default::default()) {
        Ok(_) => println!("log4rs config file is found - OK"),
        Err(error) => println!("Log config not found as \'{}\', error: \'{}\'", PATH_TO_YAML_LOG_CONFIG, error),
    }
    info!("booting up...");

    // validate incoming command line parameters
    let interface = value_t!(matches.value_of("i"), ftdi_interface).unwrap_or(ftdi_interface::INTERFACE_ANY);
    // let vid = value_t!(matches.value_of("v"), u16).unwrap_or_else(|e| { println!("vid Error = {:?}", e); e.exit() } );
    let mut vid= 0;
    if matches.is_present("v") {
        vid = ftdi_context::parse_number_str(matches.value_of("v").unwrap()).unwrap_or_default();
    }
    // let pid = value_t!(matches.value_of("p"), u16).unwrap_or_else(|e| { println!("pid Error = {:?}", e); e.exit() } );
    let mut pid = 0;
    if matches.is_present("p") {
        pid = ftdi_context::parse_number_str(matches.value_of("p").unwrap()).unwrap_or_default();
    }
    let baudrate = value_t!(matches.value_of("b"), i32).unwrap_or(115200 );
    // if tha is READ or WRITE operation ?
    let do_write = matches.is_present("w");
    let pattern_to_write = value_t!(matches.value_of("w"), u8).unwrap_or(0xff); // setup to default 255 value
    if pattern_to_write > 0xff {
        let error = FtdiError::UsbCommonError { code: -80, message: "a pattern to write should be a valid byte (u8) value".to_string(),
            backtrace: GenerateBacktrace::generate()
        };
        error!("{}", error);
        return Err(Box::new(error));
    }
    println!("Usage with values: i='{:?}', vid:pid='{:?}:{:?}', b={:?}, write='{}', w='{}'\n",
             interface, vid, pid, baudrate, do_write, pattern_to_write);

    let mut buffer:Vec<u8> = Vec::with_capacity(1024);
    if do_write {
        buffer = (0..1024).map(|_| pattern_to_write).collect();
    }

    let mut ftdi = ftdi_context::new_with_log_level(Some(4))?;
    info!("ftdi context in created - OK");

    if vid != 0 && pid != 0 && interface != ftdi_interface::INTERFACE_ANY {
        ftdi.ftdi_set_interface(ftdi_interface::INTERFACE_ANY);

        info!("start find all usb device(s)...");
        let mut ftdi_list = ftdi_device_list::new(&ftdi)?;
        let list = ftdi_list.ftdi_usb_find_all(&mut ftdi,0, 0)?;
        info!("Number of FTDI devices found: [{}] - OK", list.number_found_devices);
        info!("List of FTDI usb devices found: \'{:?}\' - OK", list.system_device_list);
        for (index, device) in list.system_device_list.iter().enumerate() {
            info!("Checking device: [{}]", index);
            let manufacturer_description = ftdi.ftdi_usb_get_strings(*device)?;
            info!("FTDI chip Manufacturer: {:?}, Description: {:?}, Serial: {:?}\n\n",
                  manufacturer_description.0, manufacturer_description.1, manufacturer_description.2);
        }
    } else {
        ftdi.ftdi_set_interface(interface);
        // Open device
        ftdi.ftdi_usb_open(vid, pid)?;
    }

    // first to check if USB was really opened
    if ftdi.usb_dev.is_some() {
        ftdi.ftdi_set_baudrate(baudrate)?;
        ftdi.ftdi_set_line_property(ftdi_bits_type::BITS_8, ftdi_stopbits_type::STOP_BIT_1, ftdi_parity_type::NONE)?;
    }

    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::SIGTERM, Arc::clone(&term))?;

    let mut write_read_result = 0;
    while !term.load(Ordering::Relaxed) && ftdi.usb_dev.is_some() /* if USB opened */ {
        // Do some time-limited stuff here
        // (if this could block forever, then there's no guarantee the signal will have any
        // effect).
        atomic::spin_loop_hint();

        if do_write {
            let size_to_write = if (baudrate / 512) > buffer.len() as i32 { buffer.len() as i32 }
                else { if (baudrate / 512) > 0 { baudrate / 512 as i32 } else { 1 as i32 } };
            let write_result = ftdi.ftdi_write_data(&mut buffer, size_to_write as u32);
            match write_result {
                Err(err) => {
                    error!("Write {:?}", err);
                    write_read_result = 0;
                },
                Ok(written_number) => {
                    debug!("written bytes = {}", written_number);
                    write_read_result = written_number;
                },
            }
        } else {
            debug!("read bytes = {}", write_read_result);
            let size_to_read = buffer.len();
            let read_result = ftdi.ftdi_read_data(&mut buffer, size_to_read);
            match read_result {
                Err(err) => {
                     error!("{}", err);
                     write_read_result = 0;
                },
                Ok(read_number) => {
                     debug!("read bytes = {}", read_number);
                     write_read_result = read_number;
                },
            }
        }
        if write_read_result < 0 {
            let sleep_millis = time::Duration::from_millis(1_000_000);
            thread::sleep(sleep_millis);
        } else if write_read_result > 0 && !do_write {
            info!("read {} bytes", write_read_result);
            // fwrite(buf, f, 1, stdout);
            // fflush(stderr);
            // fflush(stdout);
        }
    }
    debug!("got signal to exit !");
    Ok(())
}