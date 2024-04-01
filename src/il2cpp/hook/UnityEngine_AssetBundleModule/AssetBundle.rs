use std::sync::Mutex;

use fnv::FnvHashMap;
use once_cell::sync::Lazy;
use widestring::Utf16Str;

use crate::il2cpp::{
    api::il2cpp_resolve_icall,
    hook::{umamusume::{StoryRaceTextAsset, StoryTimelineData},
    Cute_UI_Assembly::AtlasReference, UnityEngine_CoreModule::Texture2D},
    types::*
};

pub const ASSET_PATH_PREFIX: &str = "assets/_gallopresources/bundle/resources/";

// *mut Il2CppObject, *mut Il2CppString
pub static REQUEST_NAMES: Lazy<Mutex<FnvHashMap<usize, usize>>> = Lazy::new(|| Mutex::new(FnvHashMap::default()));

type LoadAssetFn = extern "C" fn(this: *mut Il2CppObject, name: *mut Il2CppString, type_: *mut Il2CppReflectionType) -> *mut Il2CppObject;
extern "C" fn LoadAsset_Internal(this: *mut Il2CppObject, name: *mut Il2CppString, type_: *mut Il2CppReflectionType) -> *mut Il2CppObject {
    let mut asset = get_orig_fn!(LoadAsset_Internal, LoadAssetFn)(this, name, type_);
    on_LoadAsset(&mut asset, name);
    asset
}

type LoadAssetAsyncFn = extern "C" fn(this: *mut Il2CppObject, name: *mut Il2CppString, type_: *mut Il2CppReflectionType) -> *mut Il2CppObject;
extern "C" fn LoadAssetAsync_Internal(this: *mut Il2CppObject, name: *mut Il2CppString, type_: *mut Il2CppReflectionType) -> *mut Il2CppObject {
    let request = get_orig_fn!(LoadAssetAsync_Internal, LoadAssetAsyncFn)(this, name, type_);
    REQUEST_NAMES.lock().unwrap().insert(request as usize, name as usize); // is name even guaranteed to survive in memory..?
    request
}

type OnLoadAssetFn = fn(asset: &mut *mut Il2CppObject, name: &Utf16Str);
pub fn on_LoadAsset(asset: &mut *mut Il2CppObject, name: *mut Il2CppString) {
    let class = unsafe { (**asset).klass() };
    let name_utf16 = unsafe { (*name).to_utf16str() };
    //debug!("{} {}", unsafe { std::ffi::CStr::from_ptr((*class).name).to_str().unwrap() }, name_utf16);

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

    handler(asset, name_utf16);
}

pub fn init(_UnityEngine_AssetBundleModule: *const Il2CppImage) {
    //get_class_or_return!(UnityEngine_AssetBundleModule, UnityEngine, AssetBundle);

    let LoadAsset_Internal_addr = il2cpp_resolve_icall(
        cstr!("UnityEngine.AssetBundle::LoadAsset_Internal(System.String,System.Type)").as_ptr()
    );
    let LoadAssetAsync_Internal_addr = il2cpp_resolve_icall(
        cstr!("UnityEngine.AssetBundle::LoadAssetAsync_Internal(System.String,System.Type)").as_ptr()
    );

    new_hook!(LoadAsset_Internal_addr, LoadAsset_Internal);
    new_hook!(LoadAssetAsync_Internal_addr, LoadAssetAsync_Internal);
}