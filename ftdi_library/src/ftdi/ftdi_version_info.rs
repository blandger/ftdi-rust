use crate::ftdi::constants::{*};

/// Provide libftdi version information
/// major: Library major version
/// minor: Library minor version
/// micro: Currently unused, ight get used for hotfixes.
/// version_str: Version as (static) string
/// snapshot_str: Git snapshot version if known. Otherwise "unknown" or empty string.
#[derive(PartialEq, Eq, Debug)]
#[repr(C)]
pub struct ftdi_version_info {
    pub major: u8,
    pub minor: u8,
    pub micro: u8,
    pub version_str: String,
    pub snapshot_str: String,
}

impl ftdi_version_info {

    pub fn ftdi_get_library_version() -> ftdi_version_info  {
        ftdi_version_info {
            major: FTDI_MAJOR_VERSION,
            minor: FTDI_MINOR_VERSION,
            micro: FTDI_MICRO_VERSION,
            version_str: FTDI_VERSION_STRING.to_string(),
            snapshot_str: FTDI_SNAPSHOT_VERSION.to_string()
        }
    }
}
