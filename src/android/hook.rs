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
        hachimi.interceptor.unhook(dlopen as usize);
    }

    handle
}

type DoDlopenFn = extern "C" fn(filename: *const c_char, flags: c_int, extinfo: *const c_void, caller_addr: *const c_void) -> *mut c_void;
extern "C" fn do_dlopen(filename: *const c_char, flags: c_int, extinfo: *const c_void, caller_addr: *const c_void) -> *mut c_void {
    let hachimi = Hachimi::instance();
    let orig_fn: DoDlopenFn = unsafe {
        std::mem::transmute(hachimi.interceptor.get_trampoline_addr(do_dlopen as usize))
    };

    let handle = orig_fn(filename, flags, extinfo, caller_addr);
    if filename.is_null() {
        return handle;
    }

    let filename_str = unsafe { CStr::from_ptr(filename).to_str().unwrap() };
    if hachimi.on_dlopen(filename_str, handle as usize) {
        hachimi.interceptor.unhook(do_dlopen as usize);
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

    let force_hook_dlopen = hachimi.config.load().android.hook_libc_dlopen ||
        std::fs::metadata("/vendor/waydroid.prop").ok().is_some_and(|m| m.is_file());

    let mut dlopen_orig = libc::dlopen as usize;
    let mut dlopen_hook = dlopen as usize;
    let mut dlopen_name = "dlopen";

    const DO_DLOPEN_V24: &str = "__dl__Z9do_dlopenPKciPK17android_dlextinfoPv";  // A7, A7.1
    const DO_DLOPEN_V26: &str = "__dl__Z9do_dlopenPKciPK17android_dlextinfoPKv"; // A8 or later
    if !force_hook_dlopen {
        if api_level >= 26 {
            dlopen_orig = Interceptor::find_symbol_by_name(LINKER_MODULE, DO_DLOPEN_V26)?;
            dlopen_hook = do_dlopen as _;
            dlopen_name = DO_DLOPEN_V26;
        }
        else if api_level >= 24 {
            dlopen_orig = Interceptor::find_symbol_by_name(LINKER_MODULE, DO_DLOPEN_V24)?;
            dlopen_hook = do_dlopen as _;
            dlopen_name = DO_DLOPEN_V24;
        }
        // otherwise hook dlopen
    }

    info!("Hooking {} at {:#x}", dlopen_name, dlopen_orig);
    hachimi.interceptor.hook(dlopen_orig, dlopen_hook)?;

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