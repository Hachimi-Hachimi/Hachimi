use std::sync::atomic;

use serde::{Deserialize, Serialize};

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
        if let Err(e) = utils::kill_process_by_name(c"UnityCrashHandler64.exe") {
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

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(default = "Config::default_vsync_count")]
    pub vsync_count: i32,
    #[serde(default)]
    pub load_libraries: Vec<String>,
    #[serde(default = "Config::default_menu_open_key")]
    pub menu_open_key: u16
}

impl Config {
    fn default_vsync_count() -> i32 { -1 }
    fn default_menu_open_key() -> u16 { windows::Win32::UI::Input::KeyboardAndMouse::VK_RIGHT.0 }
}