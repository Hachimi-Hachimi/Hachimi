/* automatically generated by rust-bindgen 0.69.4 */
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

mod internal;
mod main;

pub use main::get_package_name;

use jni::sys::*;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct AppSpecializeArgs {
    pub uid: *mut jint,
    pub gid: *mut jint,
    pub gids: *mut jintArray,
    pub runtime_flags: *mut jint,
    pub rlimits: *mut jobjectArray,
    pub mount_external: *mut jint,
    pub se_info: *mut jstring,
    pub nice_name: *mut jstring,
    pub instruction_set: *mut jstring,
    pub app_data_dir: *mut jstring,
    pub fds_to_ignore: *mut jintArray,
    pub is_child_zygote: *mut jboolean,
    pub is_top_app: *mut jboolean,
    pub pkg_data_info_list: *mut jobjectArray,
    pub whitelisted_data_info_list: *mut jobjectArray,
    pub mount_data_dirs: *mut jboolean,
    pub mount_storage_dirs: *mut jboolean,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ServerSpecializeArgs {
    pub uid: *mut jint,
    pub gid: *mut jint,
    pub gids: *mut jintArray,
    pub runtime_flags: *mut jint,
    pub permitted_capabilities: *mut jlong,
    pub effective_capabilities: *mut jlong,
}
pub const Option_FORCE_DENYLIST_UNMOUNT: Option = 0;
pub const Option_DLCLOSE_MODULE_LIBRARY: Option = 1;
pub type Option = ::std::os::raw::c_int;
pub const StateFlag_PROCESS_GRANTED_ROOT: StateFlag = 1;
pub const StateFlag_PROCESS_ON_DENYLIST: StateFlag = 2;
pub type StateFlag = u32;