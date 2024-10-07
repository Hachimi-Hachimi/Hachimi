use crate::{core::Hachimi, il2cpp::{symbols::get_method_addr, types::*}};

type CheckSwapCharaFn = extern "C" fn(
    this: *mut Il2CppObject, index: i32, old_chara_id: i32, old_dress_id: i32,
    old_dress_color_id: i32, old_dress_id2: i32, old_dress_color_id2: i32, new_chara_id: i32
);
extern "C" fn CheckSwapChara(
    this: *mut Il2CppObject, index: i32, old_chara_id: i32, old_dress_id: i32,
    old_dress_color_id: i32, old_dress_id2: i32, old_dress_color_id2: i32, new_chara_id: i32
) {
    if Hachimi::instance().config.load().live_theater_allow_same_chara {
        return;
    }

    get_orig_fn!(CheckSwapChara, CheckSwapCharaFn)(
        this, index, old_chara_id, old_dress_id,
        old_dress_color_id, old_dress_id2, old_dress_color_id2, new_chara_id
    );
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, LiveTheaterCharaSelect);

    let CheckSwapChara_addr = get_method_addr(LiveTheaterCharaSelect, c"CheckSwapChara", 7);

    new_hook!(CheckSwapChara_addr, CheckSwapChara);
}