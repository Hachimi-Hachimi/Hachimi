use std::sync::atomic;

use crate::{core::Hachimi, il2cpp::{symbols::get_method_addr, types::*}};

type SetTargetFrameRateFn = extern "C" fn(value: i32);
pub extern "C" fn set_targetFrameRate(mut value: i32) {
    let target_fps = Hachimi::instance().target_fps.load(atomic::Ordering::Relaxed);
    if target_fps != -1 {
        value = target_fps;
    }
    get_orig_fn!(set_targetFrameRate, SetTargetFrameRateFn)(value);
}

pub fn init(UnityEngine_CoreModule: *const Il2CppImage) {
    get_class_or_return!(UnityEngine_CoreModule, UnityEngine, Application);

    let set_targetFrameRate_addr = get_method_addr(Application, cstr!("set_targetFrameRate"), -1);

    new_hook!(set_targetFrameRate_addr, set_targetFrameRate);
}