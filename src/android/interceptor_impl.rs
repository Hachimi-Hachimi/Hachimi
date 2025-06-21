use std::os::raw::c_void;

use crate::core::{interceptor::HookHandle, Error};

pub unsafe fn hook(orig_addr: usize, hook_addr: usize) -> Result<usize, Error> {
    Ok(dobby_rs::hook(orig_addr as *mut c_void, hook_addr as *mut c_void)? as usize)
}

impl From<dobby_rs::DobbyHookError> for Error {
    fn from(e: dobby_rs::DobbyHookError) -> Self {
        Error::HookingError(e.to_string())
    }
}

pub unsafe fn unhook(hook: &HookHandle) -> Result<(), Error> {
    dobby_rs::unhook(hook.orig_addr as *mut c_void)?;
    Ok(())
}

pub unsafe fn find_symbol_by_name(module: &str, symbol: &str) -> Result<usize, Error> {
    dobby_rs::resolve_symbol(module, symbol)
        .map(|v| v as usize)
        .ok_or(Error::SymbolNotFound(module.to_owned(), symbol.to_owned()))
}

// These are unused on Android

pub unsafe fn get_vtable_from_instance(_instance_addr: usize) -> *mut usize {
    unimplemented!();
}

pub unsafe fn hook_vtable(_vtable: *mut usize, _vtable_index: usize, _hook_addr: usize) -> Result<HookHandle, Error> {
    unimplemented!();
}

pub unsafe fn unhook_vtable(_hook: &HookHandle) -> Result<(), Error> {
    unimplemented!();
}