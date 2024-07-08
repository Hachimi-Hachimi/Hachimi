use crate::il2cpp::{hook::Plugins::AnimateToUnity::AnRoot, symbols::{create_delegate, get_method_addr}, types::*};

use super::{AsyncOperation, Object};

type UnloadUnusedAssetsFn = extern "C" fn() -> *mut Il2CppObject;
extern "C" fn UnloadUnusedAssets() -> *mut Il2CppObject {
    let res = get_orig_fn!(UnloadUnusedAssets, UnloadUnusedAssetsFn)();
    let delegate = create_delegate(unsafe { AsyncOperation::ACTION_ASYNCOPERATION_CLASS }, 1, || {
        AnRoot::OVERRIDDEN_TEXT_PARAMS.lock().unwrap().retain(|_param, gc_handle| {
            let obj = gc_handle.target();
            if obj.is_null() {
                return false;
            }
            Object::IsNativeObjectAlive(obj)
        });
    }).unwrap();
    AsyncOperation::add_completed(res, delegate);

    res
}

pub fn init(UnityEngine_CoreModule: *const Il2CppImage) {
    get_class_or_return!(UnityEngine_CoreModule, UnityEngine, Resources);

    let UnloadUnusedAssets_addr = get_method_addr(Resources, c"UnloadUnusedAssets", 0);

    new_hook!(UnloadUnusedAssets_addr, UnloadUnusedAssets);
}