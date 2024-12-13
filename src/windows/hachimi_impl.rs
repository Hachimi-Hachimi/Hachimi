use std::sync::atomic;

use serde::{Deserialize, Serialize};

use crate::{
    core::Hachimi,
    il2cpp::{
        hook::UnityEngine_CoreModule::{
            FullScreenMode_ExclusiveFullScreen, FullScreenMode_FullScreenWindow,
            QualitySettings, Screen
        },
        types::Resolution
    }
};

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

    // Apply auto full screen
    Screen::apply_auto_full_screen(Screen::get_width(), Screen::get_height());

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
    pub menu_open_key: u16,
    #[serde(default)]
    pub auto_full_screen: bool,
    #[serde(default)]
    pub full_screen_mode: FullScreenMode,
    #[serde(default)]
    pub full_screen_res: Resolution,
    #[serde(default)]
    pub resolution_scaling: ResolutionScaling,
    #[serde(default)]
    pub block_minimize_in_full_screen: bool,
    #[serde(default)]
    pub window_always_on_top: bool
}

impl Config {
    fn default_vsync_count() -> i32 { -1 }
    fn default_menu_open_key() -> u16 { windows::Win32::UI::Input::KeyboardAndMouse::VK_RIGHT.0 }
}

#[derive(Deserialize, Serialize, Copy, Clone, Default, Eq, PartialEq)]
#[repr(i32)]
pub enum FullScreenMode {
    #[default] ExclusiveFullScreen = FullScreenMode_ExclusiveFullScreen,
    FullScreenWindow = FullScreenMode_FullScreenWindow
}

#[derive(Deserialize, Serialize, Copy, Clone, Default, Eq, PartialEq)]
pub enum ResolutionScaling {
    #[default] Default,
    ScaleToScreenSize,
    ScaleToWindowSize
}

impl ResolutionScaling {
    pub fn is_not_default(&self) -> bool { *self != Self::Default }
}