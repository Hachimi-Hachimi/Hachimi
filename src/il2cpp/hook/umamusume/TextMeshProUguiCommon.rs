use crate::{core::Hachimi, il2cpp::{ext::LocalizedDataExt, hook::Unity_TextMeshPro::TMP_Text, symbols::get_method_addr, types::*}};

type AwakeFn = extern "C" fn(this: *mut Il2CppObject);
extern "C" fn Awake(this: *mut Il2CppObject) {
    get_orig_fn!(Awake, AwakeFn)(this);

    let tmp_font = Hachimi::instance().localized_data.load().load_tmp_replacement_font();
    if !tmp_font.is_null() {
        TMP_Text::set_font(this, tmp_font);
    }
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, TextMeshProUguiCommon);

    let Awake_addr = get_method_addr(TextMeshProUguiCommon, c"Awake", 0);

    new_hook!(Awake_addr, Awake);
}