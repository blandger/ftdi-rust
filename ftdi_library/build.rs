extern crate pkg_config;

fn main() {
  pkg_config::find_library("libusb-1.0").unwrap(); // see README.md - 'Install dependencies'  (if you are here)
}
