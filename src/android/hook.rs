use std::{
    ffi::CStr,
    os::raw::{c_char, c_int, c_void}
};

use jni::sys::{jint, JNINativeMethod, JNIEnv, jclass};

use crate::{android::gui_impl::input_hook, core::{Error, Hachimi, Interceptor}};
use super::utils;

const LINKER_MODULE: &str = if cfg!(target_pointer_width = "64") {
    "linker64"
} else {
    "linker"
};

type DlopenFn = extern "C" fn(filename: *const c_char, flags: c_int) -> *mut c_void;
extern "C" fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void {
    let hachimi = Hachimi::instance();
    let orig_fn: DlopenFn = unsafe {
        std::mem::transmute(hachimi.interceptor.get_trampoline_addr(dlopen as usize))
    };

    let handle = orig_fn(filename, flags);
    if filename.is_null() {
        return handle;
    }

    let filename_str = unsafe { CStr::from_ptr(filename).to_str().unwrap() };
    if hachimi.on_dlopen(filename_str, handle as usize) {
        hachimi.interceptor.unhook(loader_dlopen as usize);
    }

    handle
}

// android/platform/bionic/linker/dlfcn.cpp
type LoaderDlopenFn = extern "C" fn(filename: *const c_char, flags: c_int, caller_addr: *const c_void) -> *mut c_void;
extern "C" fn loader_dlopen(filename: *const c_char, flags: c_int, caller_addr: *const c_void) -> *mut c_void {
    let hachimi = Hachimi::instance();
    let orig_fn: LoaderDlopenFn = unsafe {
        std::mem::transmute(hachimi.interceptor.get_trampoline_addr(loader_dlopen as usize))
    };

    let handle = orig_fn(filename, flags, caller_addr);
    if filename.is_null() {
        return handle;
    }

    let filename_str = unsafe { CStr::from_ptr(filename).to_str().unwrap() };
    if hachimi.on_dlopen(filename_str, handle as usize) {
        hachimi.interceptor.unhook(loader_dlopen as usize);
    }

    handle
}

type DlopenV30Fn = extern "C" fn(filename: *const c_char, flags: c_int, extinfo: *const c_void, caller_addr: *const c_void) -> *mut c_void;
extern "C" fn dlopen_v30(filename: *const c_char, flags: c_int, extinfo: *const c_void, caller_addr: *const c_void) -> *mut c_void {
    let hachimi = Hachimi::instance();
    let orig_fn: DlopenV30Fn = unsafe {
        std::mem::transmute(hachimi.interceptor.get_trampoline_addr(dlopen_v30 as usize))
    };

    let handle = orig_fn(filename, flags, extinfo, caller_addr);
    if filename.is_null() {
        return handle;
    }

    let filename_str = unsafe { CStr::from_ptr(filename).to_str().unwrap() };
    if hachimi.on_dlopen(filename_str, handle as usize) {
        hachimi.interceptor.unhook(dlopen_v30 as usize);
    }

    handle
}

type RegisterNativesFn = extern "C" fn(env: JNIEnv, class: jclass, methods: *const JNINativeMethod, count: jint) -> jint;
#[allow(non_snake_case)]
extern "C" fn JNINativeInterface_RegisterNatives(env: JNIEnv, class: jclass, methods_: *const JNINativeMethod, count: jint) -> jint {
    let hachimi = Hachimi::instance();
    let orig_fn: RegisterNativesFn = unsafe {
        std::mem::transmute(hachimi.interceptor.get_trampoline_addr(JNINativeInterface_RegisterNatives as usize))
    };

    let methods = unsafe { std::slice::from_raw_parts(methods_, count as usize) };
    for method in methods {
        let name = unsafe { CStr::from_ptr(method.name).to_str().unwrap() };
        if name == "nativeInjectEvent" {
            info!("Got nativeInjectEvent address");
            unsafe { input_hook::NATIVE_INJECT_EVENT_ADDR = method.fnPtr as usize; };
            hachimi.interceptor.unhook(JNINativeInterface_RegisterNatives as usize);
        }
    }

    orig_fn(env, class, methods_, count)
}

fn init_internal(env: *mut jni::sys::JNIEnv) -> Result<(), Error> {
    let api_level = utils::get_device_api_level(env);
    info!("API level: {}", api_level);

    let hachimi = Hachimi::instance();

    if std::fs::metadata("/vendor/waydroid.prop").ok().is_some_and(|m| m.is_file()) {
        let dlopen_addr = libc::dlopen as usize;
        info!("Hello Waydroid! Hooking dlopen: {}", dlopen_addr);
        hachimi.interceptor.hook(dlopen_addr, dlopen as usize)?;
    }
    // A11
    else if api_level == 30 {
        let dlopen_addr = Interceptor::find_symbol_by_name(LINKER_MODULE, "__dl__Z9do_dlopenPKciPK17android_dlextinfoPKv")?;
        info!("Hooking dlopen_v30: {}", dlopen_addr);
        hachimi.interceptor.hook(dlopen_addr, dlopen_v30 as usize)?;
    }
    else {
        // A6, A7, A7.1      (api >= 23): __dl_open
        // A8, A8.1          (api >= 26): __dl__Z8__dlopenPKciPKv
        // A9, A10, A12, A13 (api >= 28): __dl___loader_dlopen
        let loader_dlopen_symbol = if api_level >= 28 {
            "__dl___loader_dlopen"
        }
        else if api_level >= 26 {
            "__dl__Z8__dlopenPKciPKv"
        }
        else {
            "__dl_open"
        };

        info!("Hooking dlopen: {}", loader_dlopen_symbol);
        let loader_dlopen_addr = Interceptor::find_symbol_by_name(LINKER_MODULE, loader_dlopen_symbol)?;
        hachimi.interceptor.hook(loader_dlopen_addr, loader_dlopen as usize)?;
    }

    if !hachimi.config.load().disable_gui {
        info!("Hooking JNINativeInterface RegisterNatives");
        let register_natives_addr = unsafe { (**env).RegisterNatives.unwrap() as usize };
        hachimi.interceptor.hook(register_natives_addr, JNINativeInterface_RegisterNatives as usize)?;
    }

    Ok(())
}

pub fn init(env: *mut jni::sys::JNIEnv) {
    init_internal(env).unwrap_or_else(|e| {
        error!("Init failed: {}", e);
        // Do nothing I guess?
    });
}