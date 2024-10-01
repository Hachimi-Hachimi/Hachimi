use widestring::Utf16Str;

use crate::{
    core::ext::Utf16StringExt,
    il2cpp::{
        api::il2cpp_resolve_icall,
        hook::{
            umamusume::FlashActionPlayer, Plugins::AnimateToUnity::AnRoot,
            UnityEngine_AssetBundleModule::AssetBundle
        },
        symbols::{get_method_addr, Array},
        types::*
    }
};

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

static mut GETCOMPONENTSINTERNAL_ADDR: usize = 0;
impl_addr_wrapper_fn!(
    GetComponentsInternal, GETCOMPONENTSINTERNAL_ADDR,
    Array<*mut Il2CppObject>,
    this: *mut Il2CppObject, type_: *mut Il2CppObject, use_search_type_as_array_return_type: bool,
    recursive: bool, include_inactive: bool, reverse: bool, /* Nullable */ result_list: *mut Il2CppObject
);

// Optimized out in assembly
pub fn GetComponentsInChildren(
    this: *mut Il2CppObject, type_: *mut Il2CppObject, include_inactive: bool
) -> Array<*mut Il2CppObject> {
    GetComponentsInternal(this, type_, true, true, include_inactive, false, 0 as _)
}

static mut SETACTIVE_ADDR: usize = 0;
impl_addr_wrapper_fn!(SetActive, SETACTIVE_ADDR, (), this: *mut Il2CppObject, value: bool);

static mut GET_ACTIVESELF_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_activeSelf, GET_ACTIVESELF_ADDR, bool, this: *mut Il2CppObject);

// hook::UnityEngine_AssetBundleModule::AssetBundle
// Generic GameObject handler for prefabs. Used for ui flash and combined ui flash
pub fn on_LoadAsset(bundle: *mut Il2CppObject, this: *mut Il2CppObject, name: &Utf16Str) {
    if !name.starts_with(AssetBundle::ASSET_PATH_PREFIX) {
        return;
    }
    let path = &name[AssetBundle::ASSET_PATH_PREFIX.len()..];

    if path.starts_with("uianimation/flash/") {
        let root = GetComponentInChildren(this, AnRoot::type_object(), false);
        if !root.is_null() {
            AnRoot::on_LoadAsset(bundle, root, name);
        }
    }
    else if path.starts_with("uianimation/flashcombine/") {
        let player = GetComponentInChildren(this, FlashActionPlayer::type_object(), false);
        if !player.is_null() {
            FlashActionPlayer::on_LoadAsset(bundle, player, name);
        }
    }
}

pub fn init(UnityEngine_CoreModule: *const Il2CppImage) {
    get_class_or_return!(UnityEngine_CoreModule, UnityEngine, GameObject);

    unsafe {
        CLASS = GameObject;
        GETCOMPONENTINCHILDREN_ADDR = get_method_addr(GameObject, c"GetComponentInChildren", 2);
        GETCOMPONENTSINTERNAL_ADDR = il2cpp_resolve_icall(
            c"UnityEngine.GameObject::GetComponentsInternal(System.Type,System.Boolean,System.Boolean,\
            System.Boolean,System.Boolean,System.Object)".as_ptr()
        );
        SETACTIVE_ADDR = il2cpp_resolve_icall(c"UnityEngine.GameObject::SetActive(System.Boolean)".as_ptr());
        GET_ACTIVESELF_ADDR = il2cpp_resolve_icall(c"UnityEngine.GameObject::get_activeSelf()".as_ptr());
    }
}