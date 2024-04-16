use std::{ffi::CString, os::raw::c_void};
use windows::Win32::Foundation::HMODULE;

use crate::windows::utils;

pub unsafe fn dlsym(handle: *mut c_void, name: &str) -> usize {
    debug_assert!(!handle.is_null());
    utils::get_proc_address(HMODULE(handle as isize), &CString::new(name).unwrap())
}