use std::{path::Path, ptr::null_mut};

use widestring::Utf16Str;

use crate::{core::{ext::{StringExt, Utf16StringExt}, Hachimi}, il2cpp::{
    api::il2cpp_object_new,
    hook::{mscorlib, UnityEngine_AssetBundleModule::AssetBundle::ASSET_PATH_PREFIX, UnityEngine_ImageConversionModule::ImageConversion},
    symbols::get_method_addr, types::*
}};

use super::TextureFormat_RGBA32;

static mut CLASS: *mut Il2CppClass = null_mut();
pub fn class() -> *mut Il2CppClass {
    unsafe { CLASS }
}

static mut CTOR_ADDR: usize = 0;
impl_addr_wrapper_fn!(_ctor, CTOR_ADDR, (),
    this: *mut Il2CppObject, width: i32, height: i32, texture_format: i32, mip_chain: bool, linear: bool
);

pub fn new(width: i32, height: i32, texture_format: i32, mip_chain: bool, linear: bool) -> *mut Il2CppObject {
    let this = il2cpp_object_new(class());
    _ctor(this, width, height, texture_format, mip_chain, linear);
    this
}

pub fn from_image_file<P: AsRef<Path>>(path: P, mip_chain: bool, mark_non_readable: bool) -> Option<*mut Il2CppObject> {
    let path_ref = path.as_ref();

    // check if file exists
    let metadata = std::fs::metadata(path_ref).ok()?;
    if !metadata.is_file() {
        return None;
    }

    // we've done everything we can, can't catch C# exceptions, yolo :)
    let path_str = path_ref.to_str()?;
    let bytes = mscorlib::File::ReadAllBytes(path_str.to_il2cpp_string());
    let texture = new(2, 2, TextureFormat_RGBA32, mip_chain, false);
    if ImageConversion::LoadImage(texture, bytes, mark_non_readable) {
        Some(texture)
    }
    else {
        warn!("Failed to load texture: {}", path_str);
        None
    }
}

pub fn load_image_file<P: AsRef<Path>>(this: *mut Il2CppObject, path: P, mark_non_readable: bool) -> bool {
    let path_ref = path.as_ref();

    // check if file exists
    let Ok(metadata) = std::fs::metadata(path_ref) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }

    // we've done everything we can, can't catch C# exceptions, yolo :)
    if let Some(path_str) = path_ref.to_str() {
        let bytes = mscorlib::File::ReadAllBytes(path_str.to_il2cpp_string());
        if ImageConversion::LoadImage(this, bytes, mark_non_readable) {
            return true;
        }
        else {
            warn!("Failed to load texture: {}", path_str);
        }
    }

    false
}

// hook::UnityEngine_AssetBundleModule::AssetBundle
pub fn on_LoadAsset(_bundle: *mut Il2CppObject, asset: &mut *mut Il2CppObject, name: &Utf16Str) {
    if !name.starts_with(ASSET_PATH_PREFIX) {
        debug!("non-resource texture: {}", name);
        return;
    }

    let orig_path = &name[ASSET_PATH_PREFIX.len()..];
    let rel_replace_path = "textures/".to_owned() + &orig_path.to_string();
    let localized_data = Hachimi::instance().localized_data.load();
    let Some(replace_path) = localized_data.get_assets_path(&rel_replace_path) else {
        return;
    };

    // TODO: match params with texture's settings
    if let Some(texture) = from_image_file(&replace_path, true, false) {
        *asset = texture;
    }
}

pub fn init(UnityEngine_CoreModule: *const Il2CppImage) {
    get_class_or_return!(UnityEngine_CoreModule, UnityEngine, Texture2D);

    unsafe {
        CLASS = Texture2D;
        CTOR_ADDR = get_method_addr(Texture2D, cstr!(".ctor"), 5);
    }
}