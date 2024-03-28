use std::sync::atomic::{AtomicI64, Ordering};

use crate::{core::Hachimi, il2cpp::{symbols::get_method_addr, types::*}};

type SetResolutionFn = extern "C" fn(w: i32, h: i32, fullscreen: bool);
extern "C" fn SetResolution(w: i32, h: i32, fullscreen: bool) {
    if !Hachimi::instance().config.load().disable_gui {
        store_resolution(w, h);
    }
    get_orig_fn!(SetResolution, SetResolutionFn)(w, h, fullscreen);
}

static RESOLUTION: AtomicI64 = AtomicI64::new(0);
fn store_resolution(w: i32, h: i32) {
    RESOLUTION.store((w as i64) << 32 | h as i64, Ordering::Relaxed);
}

pub struct Resolution(i64);
impl Resolution {
    pub fn width(&self) -> i32 {
        (self.0 >> 32) as i32
    }

    pub fn height(&self) -> i32 {
        self.0 as i32
    }
}

pub fn get_resolution() -> Resolution {
    Resolution(RESOLUTION.load(Ordering::Relaxed))
}

static mut DPI: f32 = 0.0;
pub fn get_dpi() -> f32 {
    unsafe { DPI }
}

type GetIntFn = extern "C" fn() -> i32;
type GetFloatFn = extern "C" fn() -> f32;
pub fn init(UnityEngine_CoreModule: *const Il2CppImage) {
    get_class_or_return!(UnityEngine_CoreModule, UnityEngine, Screen);

    let SetResolution_addr = get_method_addr(Screen, cstr!("SetResolution"), 3);

    new_hook!(SetResolution_addr, SetResolution);

    unsafe {
        let get_width: GetIntFn = std::mem::transmute(get_method_addr(Screen, cstr!("get_width"), -1));
        let get_height: GetIntFn = std::mem::transmute(get_method_addr(Screen, cstr!("get_height"), -1));
        let get_dpi: GetFloatFn = std::mem::transmute(get_method_addr(Screen, cstr!("get_dpi"), -1));

        store_resolution(get_width(), get_height());
        DPI = get_dpi();
    }
}