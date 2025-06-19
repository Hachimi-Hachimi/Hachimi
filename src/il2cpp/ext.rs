use std::{hash::Hasher, sync::Mutex};

use fnv::FnvHasher;
use once_cell::sync::Lazy;
use widestring::{Utf16Str, Utf16String};

use crate::core::hachimi::LocalizedData;

use super::{
    api::il2cpp_string_new_utf16,
    hook::{
        UnityEngine_AssetBundleModule::AssetBundle,
        UnityEngine_CoreModule::{HideFlags_DontUnloadUnusedAsset, Object},
        UnityEngine_TextRenderingModule::Font, Unity_TextMeshPro::TMP_FontAsset
    },
    symbols::GCHandle,
    types::*
};

pub trait StringExt {
    fn to_il2cpp_string(&self) -> *mut Il2CppString;
}

impl StringExt for str {
    fn to_il2cpp_string(&self) -> *mut Il2CppString {
        let text_utf16 = Utf16String::from_str(self);
        il2cpp_string_new_utf16(text_utf16.as_ptr(), text_utf16.len().try_into().unwrap())
    }
}

impl StringExt for String {
    fn to_il2cpp_string(&self) -> *mut Il2CppString {
        str::to_il2cpp_string(self)
    }
}

pub trait LocalizedDataExt {
    fn load_extra_asset_bundle(&self) -> *mut Il2CppObject;
    fn load_replacement_font(&self) -> *mut Il2CppObject;
    fn load_tmp_replacement_font(&self) -> *mut Il2CppObject;
}

static EXTRA_ASSET_BUNDLE_HANDLE: Lazy<Mutex<Option<GCHandle>>> = Lazy::new(|| Mutex::default());
static REPLACEMENT_FONT_HANDLE: Lazy<Mutex<Option<GCHandle>>> = Lazy::new(|| Mutex::default());
static TMP_REPLACEMENT_FONT_HANDLE: Lazy<Mutex<Option<GCHandle>>> = Lazy::new(|| Mutex::default());

impl LocalizedDataExt for LocalizedData {
    fn load_extra_asset_bundle(&self) -> *mut Il2CppObject {
        let mut handle_opt = EXTRA_ASSET_BUNDLE_HANDLE.lock().unwrap();
        if let Some(handle) = handle_opt.as_ref() {
            return handle.target();
        }

        let Some(path) = self.config.extra_asset_bundle.as_ref().map(|p| self.get_data_path(p)).unwrap_or_default() else {
            return 0 as _;
        };

        let Some(path_str) = path.to_str() else {
            error!("Invalid extra asset bundle path");
            return 0 as _;
        };

        let bundle = AssetBundle::LoadFromFile_Internal_orig(path_str.to_il2cpp_string(), 0, 0);
        if bundle.is_null() {
            error!("Failed to load extra asset bundle");
            return 0 as _;
        }

        *handle_opt = Some(GCHandle::new(bundle, false));
        bundle
    }

    fn load_replacement_font(&self) -> *mut Il2CppObject {
        let mut handle_opt = REPLACEMENT_FONT_HANDLE.lock().unwrap();
        if let Some(handle) = handle_opt.as_ref() {
            let font = handle.target();
            if Object::IsNativeObjectAlive(font) {
                return font;
            }
            else {
                debug!("Font destroyed!");
                *handle_opt = None;
            }
        }

        let Some(name) = &self.config.replacement_font_name else {
            return 0 as _;
        };

        let bundle = self.load_extra_asset_bundle();
        if bundle.is_null() {
            return 0 as _;
        }

        let font = AssetBundle::LoadAsset_Internal_orig(bundle, name.to_il2cpp_string(), Font::type_object());
        if font.is_null() {
            error!("Failed to load replacement font");
            return 0 as _;
        }
        Object::set_hideFlags(font, HideFlags_DontUnloadUnusedAsset);

        *handle_opt = Some(GCHandle::new(font, false));
        font
    }

    fn load_tmp_replacement_font(&self) -> *mut Il2CppObject {
        let mut handle_opt = TMP_REPLACEMENT_FONT_HANDLE.lock().unwrap();
        if let Some(handle) = handle_opt.as_ref() {
            let tmp_font = handle.target();
            if Object::IsNativeObjectAlive(tmp_font) {
                return tmp_font;
            }
            else {
                debug!("TMP font destroyed!");
                *handle_opt = None;
            }
        }

        let font = self.load_replacement_font();
        if font.is_null() {
            return 0 as _;
        }

        let tmp_font = TMP_FontAsset::CreateFontAsset(font);
        if tmp_font.is_null() {
            error!("Failed to create TMP font");
            return 0 as _;
        }
        Object::set_hideFlags(font, HideFlags_DontUnloadUnusedAsset);

        *handle_opt = Some(GCHandle::new(tmp_font, false));
        tmp_font
    }
}

pub trait Il2CppStringExt {
    fn chars_ptr(&self) -> *const Il2CppChar;
    fn as_utf16str(&self) -> &Utf16Str;
    fn hash(&self) -> u64;
}

impl Il2CppStringExt for Il2CppString {
    fn chars_ptr(&self) -> *const Il2CppChar {
        self.chars.as_ptr()
    }

    fn as_utf16str(&self) -> &Utf16Str {
        unsafe { Utf16Str::from_slice_unchecked(std::slice::from_raw_parts(self.chars.as_ptr(), self.length as usize)) }
    }

    fn hash(&self) -> u64 {
        let data = self.chars_ptr() as *const u8;
        let len = self.length as usize * std::mem::size_of::<Il2CppChar>();
        
        let mut hasher = FnvHasher::default();
        hasher.write(unsafe { std::slice::from_raw_parts(data, len) });
        hasher.finish()
    }
}

pub trait Il2CppObjectExt {
    fn klass(&self) -> *mut Il2CppClass;
}

impl Il2CppObjectExt for Il2CppObject {
    fn klass(&self) -> *mut Il2CppClass {
        unsafe { *self.__bindgen_anon_1.klass.as_ref() }
    }
}