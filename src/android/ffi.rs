use std::os::raw::c_int;

extern "C" {
    fn android_get_device_api_level() -> c_int;
}

pub fn get_device_api_level() -> i32 {
    unsafe { android_get_device_api_level() }
}