#![allow(non_snake_case, non_upper_case_globals)]

use windows::{core::w, Win32::System::LibraryLoader::LoadLibraryW};

use crate::windows::utils;

proxy_proc!(UnityMain, UnityMain_orig);

pub fn init() {
    unsafe {
        let Some(handle) = LoadLibraryW(w!("UnityPlayer.orig.dll")).ok().or_else(|| {
            // Try copying it
            let game_dir = utils::get_game_dir().unwrap();
            std::fs::copy(
                game_dir.join("UnityPlayer.dll"),
                game_dir.join("umamusume.exe.local\\UnityPlayer.orig.dll")
            ).ok()?;
            LoadLibraryW(w!("UnityPlayer.orig.dll")).ok()
        }) else {
            error!("Failed to load UnityPlayer.orig.dll");
            std::process::exit(1);
        };

        UnityMain_orig = utils::get_proc_address(handle, c"UnityMain");
    }
}