use crate::{core::Hachimi, il2cpp::{ext::LocalizedDataExt, symbols::{get_method_addr, get_method_overload_addr}, types::*}};

use super::TextFormat;

type GetChineseFontFn = extern "C" fn(this: *mut Il2CppObject) -> *mut Il2CppObject;
extern "C" fn GetChineseFont(this: *mut Il2CppObject) -> *mut Il2CppObject {
    let font = Hachimi::instance().localized_data.load().load_replacement_font();
    if !font.is_null() {
        return font;
    }
    get_orig_fn!(GetChineseFont, GetChineseFontFn)(this)
}

type LoadResourcesFolderFontFn = extern "C" fn(this: *mut Il2CppObject, font_type: TextFormat::Font) -> *mut Il2CppObject;
extern "C" fn LoadResourcesFolderFont(this: *mut Il2CppObject, font_type: TextFormat::Font) -> *mut Il2CppObject {
    match font_type {
        TextFormat::Font::Dynamic01 | TextFormat::Font::Chinese_Font01 => {
            let font = Hachimi::instance().localized_data.load().load_replacement_font();
            if !font.is_null() {
                return font;
            }
        }
        _ => ()
    }
    get_orig_fn!(LoadResourcesFolderFont, LoadResourcesFolderFontFn)(this, font_type)
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, TextFontManager);
    
    let GetChineseFont_addr = get_method_addr(TextFontManager, c"GetChineseFont", 0);
    let LoadResourcesFolderFont_addr = get_method_overload_addr(TextFontManager, "LoadResourcesFolderFont",
        &[Il2CppTypeEnum_IL2CPP_TYPE_VALUETYPE]);

    new_hook!(GetChineseFont_addr, GetChineseFont);
    new_hook!(LoadResourcesFolderFont_addr, LoadResourcesFolderFont);
}