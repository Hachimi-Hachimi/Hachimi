use crate::il2cpp::{symbols::get_method_addr, types::*};

use super::{Object, Sprite};

type UnloadUnusedAssetsFn = extern "C" fn() -> *mut Il2CppObject;
extern "C" fn UnloadUnusedAssets() -> *mut Il2CppObject {
    // Unity seems to destroy textures prior to calling UnloadUnusedAssets... so it's valid to do this here i guess?
    Sprite::TEXTURE_OVERRIDES.lock().unwrap().retain(|orig, replace| {
        // Destroy and remove replacement if original is dead
        let alive = Object::IsNativeObjectAlive(*orig as *mut Il2CppObject);
        if !alive {
            Object::Destroy(*replace as *mut Il2CppObject);
            debug!("texture destroyed: {}", replace);
        }

        alive
    });
    get_orig_fn!(UnloadUnusedAssets, UnloadUnusedAssetsFn)()
}

pub fn init(UnityEngine_CoreModule: *const Il2CppImage) {
    get_class_or_return!(UnityEngine_CoreModule, UnityEngine, Resources);

    let UnloadUnusedAssets_addr = get_method_addr(Resources, cstr!("UnloadUnusedAssets"), 0);

    new_hook!(UnloadUnusedAssets_addr, UnloadUnusedAssets);
}