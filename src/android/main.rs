use std::{ffi::CStr, os::raw::c_void};
use jni::{sys::jint, JavaVM};

use crate::core::Hachimi;

use super::hook;

#[allow(non_camel_case_types)]
type JniOnLoadFn = extern "C" fn(vm: JavaVM, reserved: *mut c_void) -> jint;

const LIBRARY_NAME: &CStr = cstr!("libmain_orig.so");
const JNI_ONLOAD_NAME: &CStr = cstr!("JNI_OnLoad");

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn JNI_OnLoad(vm: JavaVM, reserved: *mut c_void) -> jint {
    let orig_fn: JniOnLoadFn;
    unsafe {
        let handle = libc::dlopen(LIBRARY_NAME.as_ptr(), libc::RTLD_LAZY);
        orig_fn = std::mem::transmute(libc::dlsym(handle, JNI_ONLOAD_NAME.as_ptr()));
    }

    if !Hachimi::init() {
        return orig_fn(vm, reserved);
    }
    let env = vm.get_env().unwrap();
    hook::init(env.get_raw());

    info!("JNI_OnLoad");
    orig_fn(vm, reserved)
}