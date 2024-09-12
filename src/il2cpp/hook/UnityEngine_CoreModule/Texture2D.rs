use std::{path::Path, ptr::null_mut};

use widestring::Utf16Str;

use crate::{core::{ext::{StringExt, Utf16StringExt}, Hachimi}, il2cpp::{
    api::{il2cpp_object_new, il2cpp_resolve_icall},
    hook::{
        mscorlib,
        UnityEngine_AssetBundleModule::AssetBundle::ASSET_PATH_PREFIX,
        UnityEngine_ImageConversionModule::ImageConversion
    },
    symbols::{get_method_addr, Array},
    types::*, utils::{replace_texture_with_diff, replace_texture_with_diff_ex}
}};

use super::{Graphics, RenderTexture, Texture, TextureFormat_RGBA32};

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
    unsafe { load_image_file_unsafe(this, path, mark_non_readable) }
}

pub unsafe fn load_image_file_unsafe<P: AsRef<Path>>(this: *mut Il2CppObject, path: P, mark_non_readable: bool) -> bool {
    if let Some(path_str) = path.as_ref().to_str() {
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

pub fn render_to_texture(this: *mut Il2CppObject) -> *mut Il2CppObject {
    // Create a render texture
    let width = Texture::GetDataWidth(this);
    let height = Texture::GetDataHeight(this);
    let render_texture = RenderTexture::GetTemporary(width, height);

    // Blit this texture to the render texture
    Graphics::Blit2(this, render_texture);

    // Set the active render texture, backup the previous active texture
    let prev_active = RenderTexture::GetActive();
    RenderTexture::SetActive(render_texture);

    // Create a new texture and read the data from the render texture
    let output_texture = new(width, height, TextureFormat_RGBA32, false, false);
    ReadPixels(
        output_texture,
        Rect_t { x: 0.0, y: 0.0, width: width as f32, height: height as f32 },
        0, 0
    );

    // Revert active texture, release temp render texture
    RenderTexture::SetActive(prev_active);
    RenderTexture::ReleaseTemporary(render_texture);

    output_texture
}

// hook::UnityEngine_AssetBundleModule::AssetBundle
pub fn on_LoadAsset(_bundle: *mut Il2CppObject, this: *mut Il2CppObject, name: &Utf16Str) {
    if !name.starts_with(ASSET_PATH_PREFIX) {
        debug!("non-resource texture: {}", name);
        return;
    }

    let orig_path = &name[ASSET_PATH_PREFIX.len()..];
    let rel_replace_path = Path::new("textures").join(orig_path.to_string());
    let localized_data = Hachimi::instance().localized_data.load();
    let Some(replace_path) = localized_data.get_assets_path(&rel_replace_path) else {
        return;
    };

    // Let texture's own diff take precedence
    if replace_texture_with_diff(this, &replace_path, true) {
        return;
    }

    /**** Common diff handling ****/

    // ...chara/chrXXXX/petit/petit_chr_XXXX_YYYYYY_ZZZZ.png
    if orig_path.len() == 50 && orig_path.starts_with("chara/chr") && orig_path[13..30] == "/petit/petit_chr_" {
        let petit_type = &orig_path[42..46];
        if petit_type == "0070" || petit_type == "0071" {
            // Try to load common diff for "Train" buttons
            let rel_common_diff_path = Path::new("textures")
                .join(format!("chara/_chr/petit/petit_chr_{}.diff.png", petit_type));

            let Some(common_diff_path) = localized_data.get_assets_path(&rel_common_diff_path) else {
                return;
            };

            replace_texture_with_diff_ex(this, &replace_path, &common_diff_path, true);
        }
    }
}

static mut GETPIXELS32_ADDR: usize = 0;
impl_addr_wrapper_fn!(GetPixels32, GETPIXELS32_ADDR, Array<Color32_t>, this: *mut Il2CppObject, mip_level: i32);

static mut READPIXELS_ADDR: usize = 0;
impl_addr_wrapper_fn!(ReadPixels, READPIXELS_ADDR, (), this: *mut Il2CppObject, source: Rect_t, dest_x: i32, dest_y: i32);

pub fn init(UnityEngine_CoreModule: *const Il2CppImage) {
    get_class_or_return!(UnityEngine_CoreModule, UnityEngine, Texture2D);

    unsafe {
        CLASS = Texture2D;
        CTOR_ADDR = get_method_addr(Texture2D, c".ctor", 5);
        GETPIXELS32_ADDR = il2cpp_resolve_icall(c"UnityEngine.Texture2D::GetPixels32(System.Int32)".as_ptr());
        READPIXELS_ADDR = get_method_addr(Texture2D, c"ReadPixels", 3);
    }
}