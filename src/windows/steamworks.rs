#![allow(non_snake_case, non_upper_case_globals)]

use std::{os::raw::c_void, ptr::NonNull};

use windows::Win32::Foundation::HMODULE;

use crate::{core::Hachimi, windows::utils};

static mut SteamAPI_SteamUtils_v010_addr: usize = 0;
static mut SteamAPI_ISteamUtils_IsOverlayEnabled_addr: usize = 0;

#[repr(transparent)]
pub struct SteamUtils(NonNull<c_void>);

impl SteamUtils {
    pub fn get() -> Option<SteamUtils> {
        if unsafe { SteamAPI_SteamUtils_v010_addr } == 0 {
            return None;
        }

        let orig_fn: extern "C" fn() -> *mut c_void = unsafe {
            std::mem::transmute(SteamAPI_SteamUtils_v010_addr)
        };

        NonNull::new(orig_fn()).map(|p| Self(p))
    }

    pub fn is_overlay_enabled(&self) -> bool {
        let orig_fn: extern "C" fn(*mut c_void) -> bool = unsafe {
            std::mem::transmute(SteamAPI_ISteamUtils_IsOverlayEnabled_addr)
        };
        orig_fn(self.0.as_ptr())
    }
}

pub fn init(steam_api: HMODULE) {
    unsafe {
        SteamAPI_SteamUtils_v010_addr = utils::get_proc_address(steam_api, c"SteamAPI_SteamUtils_v010");
        SteamAPI_ISteamUtils_IsOverlayEnabled_addr = utils::get_proc_address(steam_api, c"SteamAPI_ISteamUtils_IsOverlayEnabled");
    }
}

fn is_using_overlay() -> bool {
    std::env::var("SteamOverlayGameId").is_ok()
}

pub fn is_overlay_conflicting(hachimi: &Hachimi) -> bool {
    if SteamUtils::get().is_some_and(|u| u.is_overlay_enabled()) {
        // overlay has successfully initialized and is not conflicting with Hachimi
        return false;
    }

    if !hachimi.game.is_steam_release || hachimi.config.load().disable_gui {
        return false;
    }

    is_using_overlay()
}