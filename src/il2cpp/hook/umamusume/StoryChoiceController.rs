use std::sync::atomic::{self, AtomicBool};

use crate::il2cpp::{symbols::get_method_addr, types::*};

static IS_CHECKING_CHOICE_AUTO_TAP: AtomicBool = AtomicBool::new(false);
pub fn is_checking_choice_auto_tap() -> bool {
    IS_CHECKING_CHOICE_AUTO_TAP.swap(false, atomic::Ordering::Relaxed)
}

type CheckChoiceAutoTapFn = extern "C" fn(this: *mut Il2CppObject);
extern "C" fn CheckChoiceAutoTap(this: *mut Il2CppObject) {
    IS_CHECKING_CHOICE_AUTO_TAP.store(true, atomic::Ordering::Relaxed);
    get_orig_fn!(CheckChoiceAutoTap, CheckChoiceAutoTapFn)(this);
    IS_CHECKING_CHOICE_AUTO_TAP.store(false, atomic::Ordering::Relaxed);
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, StoryChoiceController);

    let CheckChoiceAutoTap_addr = get_method_addr(StoryChoiceController, c"CheckChoiceAutoTap", 0);

    new_hook!(CheckChoiceAutoTap_addr, CheckChoiceAutoTap);
}