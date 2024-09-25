use crate::{core::Hachimi, il2cpp::{ext::LocalizedDataExt, hook::UnityEngine_UI::Text, symbols::get_method_addr, types::*}};

type AwakeFn = extern "C" fn(this: *mut Il2CppObject);
extern "C" fn Awake(this: *mut Il2CppObject) {
    get_orig_fn!(Awake, AwakeFn)(this);

    let localized_data = Hachimi::instance().localized_data.load();

    let font = localized_data.load_replacement_font();
    if !font.is_null() {
        Text::set_font(this, font);
    }

    if localized_data.config.text_common_allow_overflow {
        Text::set_horizontalOverflow(this, 1);
        Text::set_verticalOverflow(this, 1);
    }
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, TextCommon);

    let Awake_addr = get_method_addr(TextCommon, c"Awake", 0);

    new_hook!(Awake_addr, Awake);
}