use std::collections::hash_map;

use crate::il2cpp::{symbols::get_method_addr, types::*};

use super::AssetBundle::{self, REQUEST_NAMES};

type GetResultFn = extern "C" fn(this: *mut Il2CppObject) -> *mut Il2CppObject;
extern "C" fn GetResult(this: *mut Il2CppObject) -> *mut Il2CppObject {
    let mut asset = get_orig_fn!(GetResult, GetResultFn)(this);
    let name = if let hash_map::Entry::Occupied(entry) = REQUEST_NAMES.lock().unwrap().entry(this as usize) {
        entry.remove() as *mut Il2CppString
    }
    else {
        warn!("Asset bundle request not found");
        return asset;
    };
    AssetBundle::on_LoadAsset(&mut asset, name);
    asset
}

pub fn init(UnityEngine_AssetBundleModule: *const Il2CppImage) {
    get_class_or_return!(UnityEngine_AssetBundleModule, UnityEngine, AssetBundleRequest);

    let GetResult_addr = get_method_addr(AssetBundleRequest, cstr!("GetResult"), 0);

    new_hook!(GetResult_addr, GetResult);
}