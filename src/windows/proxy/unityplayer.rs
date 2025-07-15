#![allow(non_snake_case, non_upper_case_globals)]

use std::path::PathBuf;

use widestring::U16CString;
use windows::{core::PCWSTR, Win32::System::LibraryLoader::LoadLibraryW};

use crate::{core::{utils::get_file_modified_time, Hachimi}, windows::utils};

proxy_proc!(UnityMain, UnityMain_orig);

fn prepare_orig_dll() -> std::io::Result<PathBuf> {
    let src_dll = utils::get_game_dir().join("UnityPlayer.dll");
    let dest_dll = Hachimi::instance().get_data_path("UnityPlayer_orig.dll");

    if let Some(dest_mtime) = get_file_modified_time(&dest_dll) {
        if let Some(src_mtime) = get_file_modified_time(&src_dll) {
            if dest_mtime >= src_mtime {
                // Already up to date, no need to copy
                return Ok(dest_dll);
            }
        }
    }

    match std::fs::create_dir(dest_dll.parent().unwrap()) {
        Ok(()) => Ok(()),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                Ok(())
            }
            else {
                Err(e)
            }
        },
    }?;
    std::fs::copy(&src_dll, &dest_dll)?;

    Ok(dest_dll)
}

pub fn init() {
    unsafe {
        let dll_path = match prepare_orig_dll() {
            Ok(v) => v,
            Err(e) => {
                utils::show_error(format!("Failed to prepare UnityPlayer_orig.dll: {}", e));
                std::process::exit(1);
            }
        };
        let dll_path_cstr = U16CString::from_str(dll_path.to_str().unwrap()).unwrap();
        let dll_path_cstr_ptr = PCWSTR(dll_path_cstr.as_ptr());
        let handle = match LoadLibraryW(dll_path_cstr_ptr) {
            Ok(v) => v,
            Err(e) => {
                utils::show_error(format!("Failed to load UnityPlayer_orig.dll: {}", e));
                std::process::exit(1);
            }
        };

        UnityMain_orig = utils::get_proc_address(handle, c"UnityMain");
    }
}