use widestring::Utf16Str;

use crate::{core::ext::Utf16StringExt, il2cpp::{hook::Plugins::AnimateToUnity::AnRoot, symbols::get_method_addr, types::*}};

static mut CLASS: *mut Il2CppClass = 0 as _;
pub fn class() -> *mut Il2CppClass {
    unsafe { CLASS }
}

static mut GETCOMPONENTINCHILDREN_ADDR: usize = 0;
impl_addr_wrapper_fn!(
    GetComponentInChildren, GETCOMPONENTINCHILDREN_ADDR,
    *mut Il2CppObject,
    this: *mut Il2CppObject, type_: *mut Il2CppObject, include_inactive: bool
);

// hook::UnityEngine_AssetBundleModule::AssetBundle
// Generic GameObject handler for prefabs. Currently only used for ui flash (through AnRoot)
pub fn on_LoadAsset(bundle: *mut Il2CppObject, this: *mut Il2CppObject, name: &Utf16Str) {
    if name.path_filename().starts_with("pf_fl_") {
        let root = GetComponentInChildren(this, AnRoot::type_object(), false);
        if !root.is_null() {
            AnRoot::on_LoadAsset(bundle, root, name);
        }
    }
}

pub fn init(UnityEngine_CoreModule: *const Il2CppImage) {
    get_class_or_return!(UnityEngine_CoreModule, UnityEngine, GameObject);

    unsafe {
        CLASS = GameObject;
        GETCOMPONENTINCHILDREN_ADDR = get_method_addr(GameObject, c"GetComponentInChildren", 2);
    }
}