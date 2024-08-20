use crate::{core::utils::wrap_text_il2cpp, il2cpp::{symbols::get_method_addr, types::*}};

const LINE_WIDTH: i32 = 24;

type PlayFn = extern "C" fn(
    this: *mut Il2CppObject, jikkyou_text: *mut Il2CppString, jikkyou_voice_cmd: *mut Il2CppString,
    type_: i32, tension: i32, on_end: *mut Il2CppObject, is_cross_time_enable: bool
);
extern "C" fn Play(
    this: *mut Il2CppObject, mut jikkyou_text: *mut Il2CppString, jikkyou_voice_cmd: *mut Il2CppString,
    type_: i32, tension: i32, on_end: *mut Il2CppObject, is_cross_time_enable: bool
) {
    if let Some(wrapped) = wrap_text_il2cpp(jikkyou_text, LINE_WIDTH) {
        jikkyou_text = wrapped;
    }
    get_orig_fn!(Play, PlayFn)(this, jikkyou_text, jikkyou_voice_cmd, type_, tension, on_end, is_cross_time_enable);
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, JikkyoDisplay);

    let Play_addr = get_method_addr(JikkyoDisplay, c"Play", 6);

    new_hook!(Play_addr, Play);
}