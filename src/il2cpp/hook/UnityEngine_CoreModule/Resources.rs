use crate::il2cpp::{hook::Plugins::AnimateToUnity::AnMeshInfoParameterGroup, symbols::get_method_addr, types::*};

use super::Object;

type UnloadUnusedAssetsFn = extern "C" fn() -> *mut Il2CppObject;
extern "C" fn UnloadUnusedAssets() -> *mut Il2CppObject {
    // Unity seems to destroy textures prior to calling UnloadUnusedAssets... so it's valid to do this here i guess?
    AnMeshInfoParameterGroup::TEXTURE_SET_OVERRIDES.lock().unwrap().retain(|amp, overrides| {
        // Destroy replacements if the parent AnMeshParameter is dead
        let alive = Object::IsNativeObjectAlive(*amp as *mut Il2CppObject);
        if !alive {
            for (_, texture_opt) in overrides.iter() {
                if let Some(texture) = texture_opt {
                    Object::Destroy(*texture as *mut Il2CppObject);
                    debug!("amp texture destroyed: {}", texture);
                }
            }
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