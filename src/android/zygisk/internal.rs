#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use libc::{dev_t, ino_t};

use super::*;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct module_abi<T> {
    pub api_version: ::std::os::raw::c_long,
    pub impl_: *mut T,
    pub preAppSpecialize: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut T,
            arg2: *mut AppSpecializeArgs,
        ),
    >,
    pub postAppSpecialize: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut T,
            arg2: *const AppSpecializeArgs,
        ),
    >,
    pub preServerSpecialize: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut T,
            arg2: *mut ServerSpecializeArgs,
        ),
    >,
    pub postServerSpecialize: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut T,
            arg2: *const ServerSpecializeArgs,
        ),
    >,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct api_table<T> {
    pub impl_: *mut ::std::os::raw::c_void,
    pub registerModule: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut api_table<T>,
            arg2: *mut module_abi<T>,
        ) -> bool,
    >,
    pub hookJniNativeMethods: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut JNIEnv,
            arg2: *const ::std::os::raw::c_char,
            arg3: *mut JNINativeMethod,
            arg4: ::std::os::raw::c_int,
        ),
    >,
    pub pltHookRegister: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: dev_t,
            arg2: ino_t,
            arg3: *const ::std::os::raw::c_char,
            arg4: *mut ::std::os::raw::c_void,
            arg5: *mut *mut ::std::os::raw::c_void,
        ),
    >,
    pub exemptFd: ::std::option::Option<
        unsafe extern "C" fn(arg1: ::std::os::raw::c_int) -> bool,
    >,
    pub pltHookCommit: ::std::option::Option<unsafe extern "C" fn() -> bool>,
    pub connectCompanion: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut ::std::os::raw::c_void,
        ) -> ::std::os::raw::c_int,
    >,
    pub setOption: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut ::std::os::raw::c_void,
            arg2: Option,
        ),
    >,
    pub getModuleDir: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut ::std::os::raw::c_void,
        ) -> ::std::os::raw::c_int,
    >,
    pub getFlags: ::std::option::Option<
        unsafe extern "C" fn(arg1: *mut ::std::os::raw::c_void) -> u32,
    >,
}