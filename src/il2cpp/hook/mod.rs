#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

macro_rules! new_hook {
    ($orig:ident, $hook:ident) => (
        let hachimi = crate::core::Hachimi::instance();
        if !hachimi.config.load().disabled_hooks.contains(stringify!($hook)) {
            info!("new_hook!: {}", stringify!($hook));
            if ($orig != 0) {
                let res = hachimi.interceptor.hook($orig as usize, $hook as usize);
                if let Err(e) = res {
                    error!("{}", e);
                }
            }
            else {
                error!("{} is null", stringify!($orig));
            }
        }
        else {
            info!("[DISABLED] new_hook!: {}", stringify!($hook));
        }
    )
}

macro_rules! get_assembly_image_or_return {
    ($var_name:ident, $assembly_name:tt) => (
        let $var_name = match crate::il2cpp::symbols::get_assembly_image(cstr!($assembly_name)) {
            Ok(v) => v,
            Err(e) => {
                error!("{}", e);
                return;
            }
        };
    )
}

macro_rules! get_class_or_return {
    ($image:ident, $namespace:tt, $class_name:ident) => (
        let $class_name = match crate::il2cpp::symbols::get_class($image, cstr!($namespace), cstr!($class_name)) {
            Ok(v) => v,
            Err(e) => {
                error!("{}", e);
                return;
            }
        };
    )
}

macro_rules! find_nested_class_or_return {
    ($parent:ident, $class_name:ident) => (
        let $class_name = match crate::il2cpp::symbols::find_nested_class($parent, cstr!($class_name)) {
            Ok(v) => v,
            Err(e) => {
                error!("{}", e);
                return;
            }
        };
    )
}

macro_rules! impl_addr_wrapper_fn {
    ($name:tt, $addr:tt, $ret:ty, $($v:ident: $t:ty),*) => {
        pub fn $name($($v: $t),*) -> $ret {
            let orig_fn: extern "C" fn($($v: $t),*) -> $ret = unsafe { std::mem::transmute($addr) };
            orig_fn($($v),*)
        }
    };
}

pub mod mscorlib;

pub mod UnityEngine_CoreModule;
pub mod UnityEngine_AssetBundleModule;
pub mod UnityEngine_TextRenderingModule;
pub mod UnityEngine_ImageConversionModule;
pub mod UnityEngine_UI;
pub mod UnityEngine_UIModule;
pub mod Unity_TextMeshPro;

pub mod LibNative_Runtime;
pub mod umamusume;
pub mod Cute_UI_Assembly;
pub mod Plugins;
mod Cute_Cri_Assembly;
mod DOTween;

#[cfg(target_os = "android")]
mod Cute_Core_Assembly;

pub fn init() {
    info!("Initializing il2cpp hooks");

    // C# / .NET
    mscorlib::init();

    // Unity
    UnityEngine_AssetBundleModule::init();
    UnityEngine_CoreModule::init();
    UnityEngine_TextRenderingModule::init();
    UnityEngine_ImageConversionModule::init();
    UnityEngine_UI::init();
    UnityEngine_UIModule::init();
    Unity_TextMeshPro::init();

    // Umamusume
    LibNative_Runtime::init();
    umamusume::init();
    Cute_UI_Assembly::init();
    Plugins::init();
    Cute_Cri_Assembly::init();
    DOTween::init();

    #[cfg(target_os = "android")]
    Cute_Core_Assembly::init();

    info!("Hooking finished");
}