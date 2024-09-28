use crate::{core::Hachimi, il2cpp::{symbols::get_method_addr, types::*}};

use super::StoryChoiceController;

type GetTimeScaleByHighSpeedTypeFn = extern "C" fn() -> f32;
extern "C" fn GetTimeScaleByHighSpeedType() -> f32 {
    let mut res = get_orig_fn!(GetTimeScaleByHighSpeedType, GetTimeScaleByHighSpeedTypeFn)();
    if StoryChoiceController::is_checking_choice_auto_tap() {
        let delay = Hachimi::instance().config.load().story_choice_auto_select_delay;
        if delay != 0.75 {
            let mult = 0.75 / delay;
            if mult.is_finite() {
                res *= mult
            }
        }
    }

    res
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, StoryViewController);

    let GetTimeScaleByHighSpeedType_addr = get_method_addr(StoryViewController, c"GetTimeScaleByHighSpeedType", 0);

    new_hook!(GetTimeScaleByHighSpeedType_addr, GetTimeScaleByHighSpeedType);
}