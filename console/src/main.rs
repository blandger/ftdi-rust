use libusb_sys as ffi;
use std::str;
use std::ffi::CStr;

fn main() {
    let version = unsafe { ffi::libusb_get_version() };

    let rc       = str::from_utf8(unsafe { CStr::from_ptr((*version).rc)       }.to_bytes()).unwrap_or("");
    let describe = str::from_utf8(unsafe { CStr::from_ptr((*version).describe) }.to_bytes()).unwrap_or("");

    println!("libusb v{}.{}.{}.{}{} {}",
             unsafe {(*version).major}, unsafe {(*version).minor},
             unsafe {(*version).micro}, unsafe {(*version).nano},
             rc, describe);
}
