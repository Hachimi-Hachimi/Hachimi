use std::{ffi::CString, os::raw::c_void};

pub unsafe fn dlsym(handle: *mut c_void, name: &str) -> usize {
    debug_assert!(!handle.is_null());
    let name_cstr = CString::new(name).unwrap();
    libc::dlsym(handle, name_cstr.as_ptr()) as usize
}