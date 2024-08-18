use crate::{core::Hachimi, il2cpp::{symbols::get_method_addr, types::*}};

use super::{LowResolutionCamera, SingleModeStartResultCharaViewer};

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
    if mult != 1.0 &&
        !SingleModeStartResultCharaViewer::setting_up_image_effect() &&
        !LowResolutionCamera::creating_render_texture()
    {
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

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, GraphicSettings);

    let GetVirtualResolution3D_addr = get_method_addr(GraphicSettings, c"GetVirtualResolution3D", 1);
    let GetVirtualResolution_addr = get_method_addr(GraphicSettings, c"GetVirtualResolution", 0);
    let GetVirtualResolutionWidth3D_addr = get_method_addr(GraphicSettings, c"GetVirtualResolutionWidth3D", 0);

    new_hook!(GetVirtualResolution3D_addr, GetVirtualResolution3D);
    new_hook!(GetVirtualResolution_addr, GetVirtualResolution);
    new_hook!(GetVirtualResolutionWidth3D_addr, GetVirtualResolutionWidth3D);
}