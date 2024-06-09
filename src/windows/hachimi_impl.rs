use std::sync::atomic;

use crate::{core::Hachimi, il2cpp::hook::UnityEngine_CoreModule::QualitySettings};

pub fn is_il2cpp_lib(filename: &str) -> bool {
    filename == "GameAssembly.dll"
}

pub fn is_criware_lib(filename: &str) -> bool {
    filename == "cri_ware_unity.dll"
}

pub fn on_hooking_finished(hachimi: &Hachimi) {
    // Apply vsync
    if hachimi.vsync_count.load(atomic::Ordering::Relaxed) != -1 {
        QualitySettings::set_vSyncCount(1);
    }
}