use std::sync::atomic;

use crate::{core::Hachimi, il2cpp::hook::UnityEngine_CoreModule::QualitySettings};

use super::utils;

pub fn is_il2cpp_lib(filename: &str) -> bool {
    filename == "GameAssembly.dll"
}

pub fn is_criware_lib(filename: &str) -> bool {
    filename == "cri_ware_unity.dll"
}

pub fn on_hooking_finished(hachimi: &Hachimi) {
    // Kill unity crash handler (just to be safe)
    unsafe {
        if let Err(e) = utils::kill_process_by_name(cstr!("UnityCrashHandler64.exe")) {
            warn!("Error occured while trying to kill crash handler: {}", e);
        }
    };

    // Apply vsync
    if hachimi.vsync_count.load(atomic::Ordering::Relaxed) != -1 {
        QualitySettings::set_vSyncCount(1);
    }

    // Clean up the update installer
    _ = std::fs::remove_file(utils::get_tmp_installer_path());
}