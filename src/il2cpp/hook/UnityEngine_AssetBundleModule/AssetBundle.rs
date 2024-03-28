use std::sync::Mutex;

use fnv::FnvHashMap;
use once_cell::sync::Lazy;
use widestring::Utf16Str;

use crate::il2cpp::{
    hook::{umamusume::{StoryRaceTextAsset, StoryTimelineData}, Cute_UI_Assembly::AtlasReference, UnityEngine_CoreModule::Texture2D},
    symbols::get_method_addr, types::*
};

pub const ASSET_PATH_PREFIX: &str = "assets/_gallopresources/bundle/resources/";

// *mut Il2CppObject, *mut Il2CppString
pub static REQUEST_NAMES: Lazy<Mutex<FnvHashMap<usize, usize>>> = Lazy::new(|| Mutex::new(FnvHashMap::default()));

type LoadAssetFn = extern "C" fn(this: *mut Il2CppObject, name: *mut Il2CppString, type_: *mut Il2CppReflectionType) -> *mut Il2CppObject;
extern "C" fn LoadAsset(this: *mut Il2CppObject, name: *mut Il2CppString, type_: *mut Il2CppReflectionType) -> *mut Il2CppObject {
    let mut asset = get_orig_fn!(LoadAsset, LoadAssetFn)(this, name, type_);
    on_LoadAsset(&mut asset, name);
    asset
}

type LoadAssetAsyncFn = extern "C" fn(this: *mut Il2CppObject, name: *mut Il2CppString, type_: *mut Il2CppReflectionType) -> *mut Il2CppObject;
extern "C" fn LoadAssetAsync(this: *mut Il2CppObject, name: *mut Il2CppString, type_: *mut Il2CppReflectionType) -> *mut Il2CppObject {
    let request = get_orig_fn!(LoadAssetAsync, LoadAssetAsyncFn)(this, name, type_);
    REQUEST_NAMES.lock().unwrap().insert(request as usize, name as usize); // is name even guaranteed to survive in memory..?
    request
}

type OnLoadAssetFn = fn(asset: &mut *mut Il2CppObject, name: &Utf16Str);
pub fn on_LoadAsset(asset: &mut *mut Il2CppObject, name: *mut Il2CppString) {
    let class = unsafe { (**asset).klass() };
    let handler: OnLoadAssetFn = if class == StoryTimelineData::class() {
        StoryTimelineData::on_LoadAsset
    }
    else if class == Texture2D::class() {
        Texture2D::on_LoadAsset
    }
    else if class == AtlasReference::class() {
        AtlasReference::on_LoadAsset
    }
    else if class == StoryRaceTextAsset::class() {
        StoryRaceTextAsset::on_LoadAsset
    }
    else {
        return;
    };

    handler(asset, unsafe { (*name).to_utf16str() });
}

pub fn init(UnityEngine_AssetBundleModule: *const Il2CppImage) {
    get_class_or_return!(UnityEngine_AssetBundleModule, UnityEngine, AssetBundle);

    let LoadAsset_addr = get_method_addr(AssetBundle, cstr!("LoadAsset"), 2);
    let LoadAssetAsync_addr = get_method_addr(AssetBundle, cstr!("LoadAssetAsync"), 2);

    new_hook!(LoadAsset_addr, LoadAsset);
    new_hook!(LoadAssetAsync_addr, LoadAssetAsync);
}