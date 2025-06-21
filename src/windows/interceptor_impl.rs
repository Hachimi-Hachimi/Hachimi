use std::os::raw::c_void;

use crate::core::{interceptor::{HookHandle, HookType}, Error};

use minhook::MinHook;
use windows::Win32::System::Memory::{VirtualProtect, PAGE_READWRITE};

pub unsafe fn hook(orig_addr: usize, hook_addr: usize) -> Result<usize, Error> {
    let trampoline_addr = MinHook::create_hook(orig_addr as *mut c_void, hook_addr as *mut c_void)? as usize;
    MinHook::enable_hook(orig_addr as *mut c_void)?;
    Ok(trampoline_addr)
}

impl From<minhook::MH_STATUS> for Error {
    fn from(e: minhook::MH_STATUS) -> Self {
        Error::HookingError(format!("MinHook returned status: {:?}", e))
    }
}

pub unsafe fn unhook(hook: &HookHandle) -> Result<(), Error> {
    MinHook::disable_hook(hook.orig_addr as *mut c_void)?;
    MinHook::remove_hook(hook.orig_addr as *mut c_void)?;
    Ok(())
}

// Unused
pub unsafe fn find_symbol_by_name(_module: &str, _symbol: &str) -> Result<usize, Error> {
    unimplemented!();
}

pub unsafe fn get_vtable_from_instance(instance_addr: usize) -> *mut usize {
    // The address of the vtable is located right at the beginning of the object
    unsafe { *(instance_addr as *const *mut usize) }
}

pub unsafe fn hook_vtable(vtable: *mut usize, vtable_index: usize, hook_addr: usize) -> Result<HookHandle, Error> {
    let vtable_entry_addr = vtable.add(vtable_index);
    let trampoline_addr = *vtable_entry_addr;

    // Make virtual function table memory writable before modifying it
    let mut protection = PAGE_READWRITE;
    const USIZE_SIZE: usize = std::mem::size_of::<usize>();
    if VirtualProtect(vtable_entry_addr as *const c_void, USIZE_SIZE, protection, &mut protection).is_ok() {
        // Replace entry
        *vtable_entry_addr = hook_addr;
        // Revert access protection
        VirtualProtect(vtable_entry_addr as *const c_void, USIZE_SIZE, protection, &mut protection).ok();

        Ok(HookHandle {
            orig_addr: vtable_entry_addr as usize,
            trampoline_addr,
            hook_type: HookType::Vtable
        })
    }
    else {
        Err(Error::HookingError("Failed to set memory access protection".to_owned()))
    }
}

pub unsafe fn unhook_vtable(hook: &HookHandle) -> Result<(), Error> {
    // Orig addr for a vtable hook is the entry addr
    let vtable_entry_addr = hook.orig_addr as *mut usize;

    // Make virtual function table memory writable before modifying it
    let mut protection = PAGE_READWRITE;
    const USIZE_SIZE: usize = std::mem::size_of::<usize>();
    if VirtualProtect(vtable_entry_addr as *const c_void, USIZE_SIZE, protection, &mut protection).is_ok() {
        // Revert entry
        *vtable_entry_addr = hook.trampoline_addr;
        // Revert access protection
        VirtualProtect(vtable_entry_addr as *const c_void, USIZE_SIZE, protection, &mut protection).ok();

        Ok(())
    }
    else {
        Err(Error::HookingError("Failed to set memory access protection".to_owned()))
    }
}