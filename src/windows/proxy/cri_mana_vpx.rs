#![allow(non_snake_case, non_upper_case_globals)]

use widestring::U16CString;
use windows::{core::PCWSTR, Win32::System::LibraryLoader::LoadLibraryW};

use crate::{core::Hachimi, windows::utils};

proxy_proc!(criVvp9_GetAlphaInterface, criVvp9_GetAlphaInterface_orig);
proxy_proc!(criVvp9_GetInterface, criVvp9_GetInterface_orig);
proxy_proc!(criVvp9_SetUserAllocator, criVvp9_SetUserAllocator_orig);

pub fn init() {
    let dll_path = Hachimi::instance().get_data_path("cri_mana_vpx.dll");
    if !dll_path.exists() {
        warn!("cri_mana_vpx.dll doesn't exist, skipping");
        return;
    }

    unsafe {
        let dll_path_cstr = U16CString::from_str(dll_path.to_str().unwrap()).unwrap();
        let handle = LoadLibraryW(PCWSTR(dll_path_cstr.as_ptr())).expect("cri_mana_vpx.dll");

        criVvp9_GetAlphaInterface_orig = utils::get_proc_address(handle, c"criVvp9_GetAlphaInterface");
        criVvp9_GetInterface_orig = utils::get_proc_address(handle, c"criVvp9_GetInterface");
        criVvp9_SetUserAllocator_orig = utils::get_proc_address(handle, c"criVvp9_SetUserAllocator");
    }
}