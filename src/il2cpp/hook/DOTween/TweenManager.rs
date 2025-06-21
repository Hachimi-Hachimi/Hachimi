use crate::{core::Hachimi, il2cpp::{symbols::get_method_addr, types::*}};

type UpdateFn = extern "C" fn(update_type: i32, delta_time: f32, independent_time: f32);
extern "C" fn Update(update_type: i32, mut delta_time: f32, mut independent_time: f32) {
    let scale = Hachimi::instance().config.load().ui_animation_scale;
    if scale != 1.0 {
        delta_time *= scale;
        independent_time *= scale;
    }
    get_orig_fn!(Update, UpdateFn)(update_type, delta_time, independent_time);
}

pub fn init(DOTween: *const Il2CppImage) {
    get_class_or_return!(DOTween, "DG.Tweening.Core", TweenManager);

    let Update_addr = get_method_addr(TweenManager, c"Update", 3);

    new_hook!(Update_addr, Update);
}