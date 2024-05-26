use color_eyre::Report;
use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

pub fn to_wide_string(rust_string: &str) -> Vec<u16> {
    OsStr::new(rust_string)
        .encode_wide()
        .chain(Some(0))
        .collect()
}

pub fn make_lparam(low: i32, high: i32) -> isize {
    ((high as isize) << 16) | (low as isize & 0xFFFF)
}

pub fn last_os_error() -> Report {
    std::io::Error::last_os_error().into()
}
