use std::ptr::null_mut;

use widestring::Utf16Str;

use crate::{
    core::{ext::Utf16StringExt, Hachimi},
    il2cpp::{
        hook::UnityEngine_AssetBundleModule::AssetBundle::ASSET_PATH_PREFIX,
        symbols::{get_field_from_name, get_field_object_value, set_field_object_value, Array},
        ext::StringExt,
        types::*
    }
};

static mut CLASS: *mut Il2CppClass = null_mut();
pub fn class() -> *mut Il2CppClass {
    unsafe { CLASS }
}

static mut TEXT_DATA_FIELD: *mut FieldInfo = null_mut();
fn get_textData(this: *mut Il2CppObject) -> Array {
    Array::from(get_field_object_value(this, unsafe { TEXT_DATA_FIELD }))
}

// I'd move this out to its own module, but there's only a single function we need rn sooooo...
static mut KEY_TEXT_FIELD: *mut FieldInfo = null_mut();
fn Key_set_text(key: *mut Il2CppObject, value: *mut Il2CppString) {
    unsafe {
        if KEY_TEXT_FIELD.is_null() {
            KEY_TEXT_FIELD = get_field_from_name((*key).klass(), c"text");
        }
        set_field_object_value(key, KEY_TEXT_FIELD, value);
    }
}

// hook::UnityEngine_AssetBundleModule::AssetBundle
// name: assets/_gallopresources/bundle/resources/race/storyrace/text/storyrace_xxxxxxxxx.asset
pub fn on_LoadAsset(_bundle: *mut Il2CppObject, this: *mut Il2CppObject, name: &Utf16Str) {
    if !name.starts_with(ASSET_PATH_PREFIX) {
        // ???
        return;
    }

    let base_path = name[ASSET_PATH_PREFIX.len()..].path_basename();
    let dict_path = base_path.to_string() + ".json";
    let localized_data = Hachimi::instance().localized_data.load();
    let Some(dict): Option<Vec<String>> = localized_data.load_assets_dict(Some(&dict_path)) else {
        return;
    };

    let text_data = get_textData(this);
    for (i, key) in unsafe { text_data.as_slice().iter().enumerate() } {
        let Some(text) = dict.get(i) else { continue };
        Key_set_text(*key, text.to_il2cpp_string());
    }
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, StoryRaceTextAsset);

    unsafe {
        CLASS = StoryRaceTextAsset;
        TEXT_DATA_FIELD = get_field_from_name(StoryRaceTextAsset, c"textData")
    }
}