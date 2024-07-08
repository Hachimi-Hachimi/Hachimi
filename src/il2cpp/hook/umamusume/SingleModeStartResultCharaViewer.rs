use std::sync::atomic::{self, AtomicBool};

use crate::il2cpp::{symbols::get_method_addr, types::*};

static SETTING_UP_IMAGE_EFFECT: AtomicBool = AtomicBool::new(false);
pub fn setting_up_image_effect() -> bool {
    SETTING_UP_IMAGE_EFFECT.load(atomic::Ordering::Relaxed)
}

type SetupImageEffectFn = extern "C" fn(this: *mut Il2CppObject);
extern "C" fn SetupImageEffect(this: *mut Il2CppObject) {
    SETTING_UP_IMAGE_EFFECT.store(true, atomic::Ordering::Relaxed);
    get_orig_fn!(SetupImageEffect, SetupImageEffectFn)(this);
    SETTING_UP_IMAGE_EFFECT.store(false, atomic::Ordering::Relaxed);
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, SingleModeStartResultCharaViewer);

    let SetupImageEffect_addr = get_method_addr(SingleModeStartResultCharaViewer, c"SetupImageEffect", 0);

    new_hook!(SetupImageEffect_addr, SetupImageEffect);
}