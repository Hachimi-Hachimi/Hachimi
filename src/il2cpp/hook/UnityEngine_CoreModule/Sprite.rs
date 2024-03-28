use std::sync::Mutex;

use fnv::FnvHashMap;
use once_cell::sync::Lazy;

use crate::il2cpp::{symbols::get_method_addr, types::*};

// *mut Il2CppObject, *mut Il2CppObject
// The textures are destroyed in the Resources::UnloadUnusedAssets hook.
pub static TEXTURE_OVERRIDES: Lazy<Mutex<FnvHashMap<usize, usize>>> = Lazy::new(|| Mutex::new(FnvHashMap::default()));

type GetTextureFn = extern "C" fn(this: *mut Il2CppObject) -> *mut Il2CppObject;
pub extern "C" fn get_texture(this: *mut Il2CppObject) -> *mut Il2CppObject {
    let orig = get_orig_fn!(get_texture, GetTextureFn)(this);
    if let Some(replace) = TEXTURE_OVERRIDES.lock().unwrap().get(&(orig as usize)) {
        *replace as *mut Il2CppObject
    }
    else {
        orig
    }
}

pub fn orig_get_texture(this: *mut Il2CppObject) -> *mut Il2CppObject {
    get_orig_fn!(get_texture, GetTextureFn)(this)
}

pub fn init(UnityEngine_CoreModule: *const Il2CppImage) {
    get_class_or_return!(UnityEngine_CoreModule, UnityEngine, Sprite);

    let get_texture_addr = get_method_addr(Sprite, cstr!("get_texture"), 0);

    new_hook!(get_texture_addr, get_texture);
}