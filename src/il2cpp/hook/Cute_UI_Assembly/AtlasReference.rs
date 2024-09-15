use std::ptr::null_mut;

use widestring::Utf16Str;

use crate::{core::{ext::Utf16StringExt, Hachimi}, il2cpp::{
    hook::{
        UnityEngine_AssetBundleModule::AssetBundle,
        UnityEngine_CoreModule::Sprite
    },
    symbols::{get_field_from_name, get_field_object_value, Array},
    types::*, utils::replace_texture_with_diff
}};

static mut CLASS: *mut Il2CppClass = null_mut();
pub fn class() -> *mut Il2CppClass {
    unsafe { CLASS }
}

static mut SPRITES_FIELD: *mut FieldInfo = null_mut();
fn get_sprites(this: *mut Il2CppObject) -> Array {
    Array::from(get_field_object_value(this, unsafe { SPRITES_FIELD }))
}

// hook::UnityEngine_AssetBundleModule::AssetBundle
// name: assets/_gallopresources/bundle/resources/atlas/**.asset
pub fn on_LoadAsset(bundle: *mut Il2CppObject, this: *mut Il2CppObject, name: &Utf16Str) {
    if !name.starts_with(AssetBundle::ASSET_PATH_PREFIX) {
        debug!("non-resource atlas: {}", name);
        return;
    }

    let base_path = name[AssetBundle::ASSET_PATH_PREFIX.len()..].path_basename();
    if !base_path.starts_with("atlas/") {
        debug!("bad path: {}", name);
        return;
    }
    let rel_replace_path = base_path.to_string() + ".png";
    let localized_data = Hachimi::instance().localized_data.load();
    let Some(replace_path) = localized_data.get_assets_path(&rel_replace_path) else {
        return;
    };
    let metadata = localized_data.load_asset_metadata(&rel_replace_path);
    if !AssetBundle::check_asset_bundle_name(bundle, &metadata) {
        return;
    }

    // All of the sprites in the atlas uses the same texture so we just need to replace one of them
    let sprites = get_sprites(this);
    if let Some(sprite) = unsafe { sprites.as_slice().get(0) } {
        replace_texture_with_diff(Sprite::get_texture(*sprite), replace_path, true);
    }
}

pub fn init(Cute_UI_Assembly: *const Il2CppImage) {
    get_class_or_return!(Cute_UI_Assembly, "Cute.UI", AtlasReference);

    unsafe {
        CLASS = AtlasReference;
        SPRITES_FIELD = get_field_from_name(AtlasReference, c"sprites")
    }
}