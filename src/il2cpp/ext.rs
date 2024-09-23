use std::sync::Mutex;

use once_cell::sync::Lazy;
use widestring::Utf16String;

use crate::core::hachimi::LocalizedData;

use super::{api::il2cpp_string_new_utf16, hook::UnityEngine_AssetBundleModule::AssetBundle, symbols::GCHandle, types::*};

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

trait LocalizedDataExt {
    fn load_extra_asset_bundle(&self) -> *mut Il2CppObject;
}

static EXTRA_ASSET_BUNDLE_HANDLE: Lazy<Mutex<Option<GCHandle>>> = Lazy::new(|| Mutex::default());

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
            return 0 as _;
        };

        let bundle = AssetBundle::LoadFromFile_Internal_orig(path_str.to_il2cpp_string(), 0, 0);
        if bundle.is_null() {
            return 0 as _;
        }

        *handle_opt = Some(GCHandle::new(bundle, false));
        bundle
    }
}