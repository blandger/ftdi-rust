#![allow(dead_code)]

use std::path::Path;
use std::{env, fs};

// const SETTINGS_FILE: &str = "Settings.toml";
const LOG4RS_FILE: &str = "log4rs.yaml";
fn main() {
    let target_dir_path = env::var("OUT_DIR").unwrap();
    println!("Out dir = {}", target_dir_path);
    copy_to_examples(&target_dir_path, LOG4RS_FILE);
    // copy(&target_dir_path, SETTINGS_FILE);
}

fn copy_to_examples<S: AsRef<std::ffi::OsStr> + ?Sized, P: Copy + AsRef<Path>>(target_dir_path: &S, file_name: P) {
    let path_to_target = Path::new(
        &target_dir_path).join("../../../examples").as_path()
        // .join("examples/")
        .join(file_name);
    println!("Out \'examples\' target dir = {:?}", path_to_target);
    fs::copy(file_name, path_to_target).unwrap();
}

fn copy_to_debug<S: AsRef<std::ffi::OsStr> + ?Sized, P: Copy + AsRef<Path>>(target_dir_path: &S, file_name: P) {
    let path_to_target = Path::new(
        &target_dir_path).join("../../..").as_path()
        // .join("examples/")
        .join(file_name);
    println!("Out \'target debug\' dir = {:?}", path_to_target);
    fs::copy(file_name, path_to_target).unwrap();
}

