#![allow(non_snake_case, non_upper_case_globals)]

use widestring::U16CString;
use windows::{core::PCWSTR, Win32::System::LibraryLoader::LoadLibraryW};

use crate::{core::Hachimi, windows::utils};

proxy_proc!(UnityMain, UnityMain_orig);

pub fn init() {
    unsafe {
        let dll_path = Hachimi::instance().get_data_path("UnityPlayer_orig.dll");
        let dll_path_cstr = U16CString::from_str(dll_path.to_str().unwrap()).unwrap();
        let dll_path_cstr_ptr = PCWSTR(dll_path_cstr.as_ptr());
        let Some(handle) = LoadLibraryW(dll_path_cstr_ptr).ok().or_else(|| {
            // Try copying it
            let game_dir = utils::get_game_dir().unwrap();
            std::fs::copy(game_dir.join("UnityPlayer.dll"), dll_path)
                .inspect_err(|e| error!("{}", e))
                .ok()?;
            LoadLibraryW(dll_path_cstr_ptr).ok()
        }) else {
            error!("Failed to load UnityPlayer_orig.dll");
            std::process::exit(1);
        };

        UnityMain_orig = utils::get_proc_address(handle, c"UnityMain");
    }
}