use std::collections::hash_map;

use crate::il2cpp::{symbols::get_method_addr, types::*};

use super::AssetBundle::{self, REQUEST_INFOS};

type GetResultFn = extern "C" fn(this: *mut Il2CppObject) -> *mut Il2CppObject;
extern "C" fn GetResult(this: *mut Il2CppObject) -> *mut Il2CppObject {
    let asset = get_orig_fn!(GetResult, GetResultFn)(this);
    let info = if let hash_map::Entry::Occupied(entry) = REQUEST_INFOS.lock().unwrap().entry(this as usize) {
        entry.remove()
    }
    else {
        warn!("Asset bundle request not found");
        return asset;
    };
    AssetBundle::on_LoadAsset(info.bundle as _, asset, info.name());
    asset
}

pub fn init(UnityEngine_AssetBundleModule: *const Il2CppImage) {
    get_class_or_return!(UnityEngine_AssetBundleModule, UnityEngine, AssetBundleRequest);

    let GetResult_addr = get_method_addr(AssetBundleRequest, c"GetResult", 0);

    new_hook!(GetResult_addr, GetResult);
}