use crate::{core::Hachimi, il2cpp::{ext::LocalizedDataExt, symbols::get_method_addr, types::*}};

type GetFontFn = extern "C" fn(this: *mut Il2CppObject, font_name: *mut Il2CppString) -> *mut Il2CppObject;
extern "C" fn _GetFont(this: *mut Il2CppObject, font_name: *mut Il2CppString) -> *mut Il2CppObject {
    let font = Hachimi::instance().localized_data.load().load_replacement_font();
    if !font.is_null() {
        return font;
    }
    get_orig_fn!(_GetFont, GetFontFn)(this, font_name)
}

type GetFontFromCommonFn = extern "C" fn(this: *mut Il2CppObject, font_name: *mut Il2CppString)  -> *mut Il2CppObject;
extern "C" fn _GetFontFromCommon(this: *mut Il2CppObject, font_name: *mut Il2CppString) -> *mut Il2CppObject {
    let font = Hachimi::instance().localized_data.load().load_replacement_font();
    if !font.is_null() {
        return font;
    }
    get_orig_fn!(_GetFontFromCommon, GetFontFromCommonFn)(this, font_name)
}

pub fn init(Plugins: *const Il2CppImage) {
    get_class_or_return!(Plugins, AnimateToUnity, AnGlobalData);

    let _GetFont_addr = get_method_addr(AnGlobalData, c"_GetFont", 1);
    let _GetFontFromCommon_addr = get_method_addr(AnGlobalData, c"_GetFontFromCommon", 1);

    new_hook!(_GetFont_addr, _GetFont);
    new_hook!(_GetFontFromCommon_addr, _GetFontFromCommon);
}