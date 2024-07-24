use serde::Deserialize;
use widestring::Utf16Str;

use crate::{
    core::{ext::Utf16StringExt, hachimi::AssetInfo, Hachimi},
    il2cpp::{
        api::{il2cpp_class_get_type, il2cpp_type_get_object},
        hook::{
            Plugins::AnimateToUnity::AnRoot, UnityEngine_AssetBundleModule::AssetBundle,
            UnityEngine_CoreModule::GameObject
        },
        symbols::{get_field_from_name, get_field_object_value},
        types::*
    }
};

static mut TYPE_OBJECT: *mut Il2CppObject = 0 as _;
pub fn type_object() -> *mut Il2CppObject {
    unsafe { TYPE_OBJECT }
}

// GameObject
static mut _FLASHPREFAB_FIELD: *mut FieldInfo = 0 as _;
pub fn get__flashPrefab(this: *mut Il2CppObject) -> *mut Il2CppObject {
    get_field_object_value(this, unsafe { _FLASHPREFAB_FIELD })
}

#[derive(Deserialize)]
pub struct FlashActionPlayerData {
    an_root: Option<AnRoot::AnRootData>
}

pub fn on_LoadAsset(bundle: *mut Il2CppObject, this: *mut Il2CppObject, name: &Utf16Str) {
    // SAFETY: The asset path has been checked prior to this being called in GameObject::on_LoadAsset
    let base_path = name[AssetBundle::ASSET_PATH_PREFIX.len()..].path_basename();

    let localized_data = Hachimi::instance().localized_data.load();
    let asset_info: AssetInfo<FlashActionPlayerData> = localized_data.load_asset_info(&base_path.to_string());
    if !AssetBundle::check_asset_bundle_name(bundle, asset_info.metadata_ref()) {
        return;
    }

    let flash_prefab = get__flashPrefab(this);
    if flash_prefab.is_null() {
        return;
    }

    let root = GameObject::GetComponentInChildren(flash_prefab, AnRoot::type_object(), false);
    if !root.is_null() {
        AnRoot::patch_asset(root, asset_info.data.map(|d| d.an_root).unwrap_or_default().as_ref());
    }
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, FlashActionPlayer);

    unsafe {
        TYPE_OBJECT = il2cpp_type_get_object(il2cpp_class_get_type(FlashActionPlayer));
        _FLASHPREFAB_FIELD = get_field_from_name(FlashActionPlayer, c"_flashPrefab");
    }
}