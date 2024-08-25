use crate::{
    core::{utils::scale_to_aspect_ratio, Hachimi},
    il2cpp::{api::il2cpp_resolve_icall, symbols::get_method_addr, types::*},
};

static mut GET_CURRENTRESOLUTION_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_currentResolution, GET_CURRENTRESOLUTION_ADDR, Resolution,);

static mut GET_WIDTH_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_width, GET_WIDTH_ADDR, i32,);

static mut GET_HEIGHT_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_height, GET_HEIGHT_ADDR, i32,);

static mut GET_FULLSCREEN_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_fullScreen, GET_FULLSCREEN_ADDR, bool,);

pub fn apply_auto_full_screen(mut width: i32, mut height: i32) -> bool {
    let windows_config = &Hachimi::instance().config.load().windows;
    let preferred_res = &windows_config.full_screen_res;
    let (preferred_width, preferred_height) = if preferred_res.width > 0 && preferred_res.height > 0 {
        (preferred_res.width, preferred_res.height)
    }
    else {
        let res = get_currentResolution();
        (res.width, res.height)
    };

    if width > 0 && height > 0 && (width > height) == (preferred_width > preferred_height) {
        let aspect_ratio = width as f32 / height as f32;
        (width, height) = scale_to_aspect_ratio((preferred_width, preferred_height), aspect_ratio, false)
    }
    else {
        return false;
    }

    let full_screen_mode = windows_config.full_screen_mode.value();
    let preferred_refresh_rate = preferred_res.refresh_rate;
    get_orig_fn!(SetResolution, SetResolutionFn)(width, height, full_screen_mode, preferred_refresh_rate);

    true
}

type SetResolutionFn = extern "C" fn(width: i32, height: i32, fullscreen_mode: i32, preferred_refresh_rate: i32);
extern "C" fn SetResolution(width: i32, height: i32, full_screen_mode: i32, preferred_refresh_rate: i32) {
    let windows_config = &Hachimi::instance().config.load().windows;
    if windows_config.auto_full_screen {
        if apply_auto_full_screen(width, height) {
            return;
        }
    }

    get_orig_fn!(SetResolution, SetResolutionFn)(width, height, full_screen_mode, preferred_refresh_rate);
}

pub fn init(UnityEngine_CoreModule: *const Il2CppImage) {
    get_class_or_return!(UnityEngine_CoreModule, UnityEngine, Screen);

    let SetResolution_addr = il2cpp_resolve_icall(
        c"UnityEngine.Screen::SetResolution(System.Int32,System.Int32,UnityEngine.FullScreenMode,System.Int32)".as_ptr()
    );

    new_hook!(SetResolution_addr, SetResolution);

    unsafe {
        GET_CURRENTRESOLUTION_ADDR = get_method_addr(Screen, c"get_currentResolution", 0);
        GET_WIDTH_ADDR = il2cpp_resolve_icall(c"UnityEngine.Screen::get_width()".as_ptr());
        GET_HEIGHT_ADDR = il2cpp_resolve_icall(c"UnityEngine.Screen::get_height()".as_ptr());
        GET_FULLSCREEN_ADDR =il2cpp_resolve_icall(c"UnityEngine.Screen::get_fullScreen()".as_ptr());
    }
}