use serde::{Deserialize, Serialize};

use crate::{core::Hachimi, il2cpp::{symbols::get_method_addr, types::*}};

use super::SingleModeStartResultCharaViewer;

#[cfg(target_os = "windows")]
static mut CLASS: *mut Il2CppClass = 0 as _;
#[cfg(target_os = "windows")]
pub fn class() -> *mut Il2CppClass {
    unsafe { CLASS }
}

#[cfg(target_os = "windows")]
pub fn instance() -> *mut Il2CppObject {
    let Some(singleton) = crate::il2cpp::symbols::SingletonLike::new(class()) else {
        return 0 as _;
    };
    singleton.instance()
}

type GetVirtualResolutionFn = extern "C" fn(this: *mut Il2CppObject) -> Vector2Int_t;
extern "C" fn GetVirtualResolution(this: *mut Il2CppObject) -> Vector2Int_t {
    let mut res = get_orig_fn!(GetVirtualResolution, GetVirtualResolutionFn)(this);
    let mult = Hachimi::instance().config.load().virtual_res_mult;
    if mult != 1.0 {
        res *= mult;
    }
    res
}

type GetVirtualResolution3DFn = extern "C" fn(this: *mut Il2CppObject, is_forced_wide_aspect: bool) -> Vector2Int_t;
extern "C" fn GetVirtualResolution3D(this: *mut Il2CppObject, is_forced_wide_aspect: bool) -> Vector2Int_t {
    let mut res = get_orig_fn!(GetVirtualResolution3D, GetVirtualResolution3DFn)(this, is_forced_wide_aspect);
    let mult = Hachimi::instance().config.load().virtual_res_mult;
    if mult != 1.0 && !SingleModeStartResultCharaViewer::setting_up_image_effect() {
        res *= mult;
    }
    res
}

type GetVirtualResolutionWidth3DFn = extern "C" fn(this: *mut Il2CppObject) -> i32;
extern "C" fn GetVirtualResolutionWidth3D(this: *mut Il2CppObject) -> i32 {
    let mut width = get_orig_fn!(GetVirtualResolutionWidth3D, GetVirtualResolutionWidth3DFn)(this);
    let mult = Hachimi::instance().config.load().virtual_res_mult;
    if mult != 1.0 {
        width = (width as f32 * mult) as i32;
    }
    width
}

#[cfg(target_os = "windows")]
static mut UPDATE3DRENDERTEXTURE_ADDR: usize = 0;
#[cfg(target_os = "windows")]
impl_addr_wrapper_fn!(Update3DRenderTexture, UPDATE3DRENDERTEXTURE_ADDR, (), this: *mut Il2CppObject);

#[derive(Default, Copy, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[repr(i32)]
pub enum GraphicsQuality {
    #[default] Default = -1,
    Toon1280 = 0,
    Toon1280x2,
    Toon1280x4,
    ToonFull,
    Max
}

type ApplyGraphicsQualityFn = extern "C" fn(this: *mut Il2CppObject, quality: GraphicsQuality, force: bool);
extern "C" fn ApplyGraphicsQuality(this: *mut Il2CppObject, quality: GraphicsQuality, force: bool) {
    let custom_quality = Hachimi::instance().config.load().graphics_quality;
    if custom_quality != GraphicsQuality::Default {
        return get_orig_fn!(ApplyGraphicsQuality, ApplyGraphicsQualityFn)(this, custom_quality, true);
    }

    get_orig_fn!(ApplyGraphicsQuality, ApplyGraphicsQualityFn)(this, quality, force);
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, GraphicSettings);

    let GetVirtualResolution3D_addr = get_method_addr(GraphicSettings, c"GetVirtualResolution3D", 1);
    let GetVirtualResolution_addr = get_method_addr(GraphicSettings, c"GetVirtualResolution", 0);
    let GetVirtualResolutionWidth3D_addr = get_method_addr(GraphicSettings, c"GetVirtualResolutionWidth3D", 0);
    let ApplyGraphicsQuality_addr = get_method_addr(GraphicSettings, c"ApplyGraphicsQuality", 2);

    new_hook!(GetVirtualResolution3D_addr, GetVirtualResolution3D);
    new_hook!(GetVirtualResolution_addr, GetVirtualResolution);
    new_hook!(GetVirtualResolutionWidth3D_addr, GetVirtualResolutionWidth3D);
    new_hook!(ApplyGraphicsQuality_addr, ApplyGraphicsQuality);

    #[cfg(target_os = "windows")]
    unsafe {
        CLASS = GraphicSettings;
        UPDATE3DRENDERTEXTURE_ADDR = get_method_addr(GraphicSettings, c"Update3DRenderTexture", 0);
    }
}