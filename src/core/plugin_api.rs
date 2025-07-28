use std::ffi::{c_char, c_void, CStr};

use crate::{core::{Hachimi, Interceptor}, il2cpp};

pub type HachimiInitFn = extern "C" fn(vtable: *const Vtable);

unsafe extern "C" fn hachimi_instance() -> *const Hachimi {
    Hachimi::instance().as_ref()
}

unsafe extern "C" fn hachimi_get_interceptor(this: *const Hachimi) -> *const Interceptor {
    &(*this).interceptor
}

unsafe extern "C" fn interceptor_hook(this: *const Interceptor, orig_addr: *mut c_void, hook_addr: *mut c_void) -> *mut c_void {
    (*this).hook(orig_addr as _, hook_addr as _).unwrap_or(0) as _
}

unsafe extern "C" fn interceptor_hook_vtable(this: *const Interceptor, vtable: *mut *mut c_void, vtable_index: *mut c_void, hook_addr: *mut c_void) -> *mut c_void {
    (*this).hook_vtable(vtable as _, vtable_index as _, hook_addr as _).unwrap_or(0) as _
}

unsafe extern "C" fn interceptor_get_trampoline_addr(this: *const Interceptor, hook_addr: *mut c_void) -> *mut c_void {
    (*this).get_trampoline_addr(hook_addr as _) as _
}

unsafe extern "C" fn interceptor_unhook(this: *const Interceptor, hook_addr: *mut c_void, out_orig_addr: *mut *mut c_void) -> bool {
    if let Some(handle) = (*this).unhook(hook_addr as _) {
        if !out_orig_addr.is_null() {
            *out_orig_addr = handle.orig_addr as _;
        }
        true
    }
    else {
        false
    }
}

unsafe extern "C" fn il2cpp_resolve_symbol(name: *const c_char) -> *mut c_void {
    let Ok(name) = CStr::from_ptr(name).to_str() else {
        return 0 as _;
    };
    il2cpp::symbols::dlsym(name) as _
}

unsafe extern "C" fn log(level: i32, target: *const c_char, message: *const c_char) {
    let target = CStr::from_ptr(target).to_string_lossy();
    let message = CStr::from_ptr(message).to_string_lossy();
    let level = match level {
        1 => log::Level::Error,
        2 => log::Level::Warn,
        3 => log::Level::Info,
        4 => log::Level::Debug,
        5 => log::Level::Trace,

        _ => log::Level::Info
    };
    log!(target: &target, level, "{}", message);
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vtable {
    hachimi_instance: unsafe extern "C" fn() -> *const Hachimi,
    hachimi_get_interceptor: unsafe extern "C" fn(this: *const Hachimi) -> *const Interceptor,
    interceptor_hook: unsafe extern "C" fn(this: *const Interceptor, orig_addr: *mut c_void, hook_addr: *mut c_void) -> *mut c_void,
    interceptor_hook_vtable: unsafe extern "C" fn(this: *const Interceptor, vtable: *mut *mut c_void, vtable_index: *mut c_void, hook_addr: *mut c_void) -> *mut c_void,
    interceptor_get_trampoline_addr: unsafe extern "C" fn(this: *const Interceptor, hook_addr: *mut c_void) -> *mut c_void,
    interceptor_unhook: unsafe extern "C" fn(this: *const Interceptor, hook_addr: *mut c_void, out_orig_addr: *mut *mut c_void) -> bool,
    il2cpp_resolve_symbol: unsafe extern "C" fn(name: *const c_char) -> *mut c_void,
    log: unsafe extern "C" fn(level: i32, target: *const c_char, message: *const c_char)
}

impl Vtable {
    pub const VALUE: Self = Self {
        hachimi_instance,
        hachimi_get_interceptor,
        interceptor_hook,
        interceptor_hook_vtable,
        interceptor_get_trampoline_addr,
        interceptor_unhook,
        il2cpp_resolve_symbol,
        log
    };

    pub fn instantiate() -> Self {
        Self::VALUE.clone()
    }
}