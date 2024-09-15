use crate::il2cpp::{symbols::get_method_addr, types::*};

#[cfg(target_os = "android")]
use crate::core::Hachimi;

#[cfg(target_os = "android")]
extern "C" fn ChangeScreenOrientationLandscapeAsync_MoveNext(enumerator: *mut Il2CppObject) -> bool {
    use crate::il2cpp::symbols::MoveNextFn;
    let moved = get_orig_fn!(ChangeScreenOrientationLandscapeAsync_MoveNext, MoveNextFn)(enumerator);
    if !moved {
        super::UIManager::apply_ui_scale();
    }
    moved
}

#[cfg(target_os = "android")]
extern "C" fn ChangeScreenOrientationPortraitAsync_MoveNext(enumerator: *mut Il2CppObject) -> bool {
    use crate::il2cpp::symbols::MoveNextFn;
    let moved = get_orig_fn!(ChangeScreenOrientationPortraitAsync_MoveNext, MoveNextFn)(enumerator);
    if !moved {
        super::UIManager::apply_ui_scale();
    }
    moved
}

#[cfg(target_os = "android")]
type ChangeScreenOrientationLandscapeAsyncFn = extern "C" fn() -> crate::il2cpp::symbols::IEnumerator;
#[cfg(target_os = "android")]
extern "C" fn ChangeScreenOrientationLandscapeAsync() -> crate::il2cpp::symbols::IEnumerator {
    let enumerator = get_orig_fn!(ChangeScreenOrientationLandscapeAsync, ChangeScreenOrientationLandscapeAsyncFn)();
    if Hachimi::instance().config.load().ui_scale == 1.0 { return enumerator; }

    if let Err(e) = enumerator.hook_move_next(ChangeScreenOrientationLandscapeAsync_MoveNext) {
        error!("Failed to hook enumerator: {}", e);
    }

    enumerator
}

#[cfg(target_os = "android")]
type ChangeScreenOrientationPortraitAsyncFn = extern "C" fn() -> crate::il2cpp::symbols::IEnumerator;
#[cfg(target_os = "android")]
extern "C" fn ChangeScreenOrientationPortraitAsync() -> crate::il2cpp::symbols::IEnumerator {
    let enumerator = get_orig_fn!(ChangeScreenOrientationPortraitAsync, ChangeScreenOrientationPortraitAsyncFn)();
    if Hachimi::instance().config.load().ui_scale == 1.0 { return enumerator; }

    if let Err(e) = enumerator.hook_move_next(ChangeScreenOrientationPortraitAsync_MoveNext) {
        error!("Failed to hook enumerator: {}", e);
    }

    enumerator
}

#[cfg(target_os = "windows")]
type GetWidthFn = extern "C" fn() -> i32;
#[cfg(target_os = "windows")]
extern "C" fn get_Width() -> i32 {
    if let Some((width, _)) = crate::windows::utils::get_scaling_res() {
        return width;
    }

    get_orig_fn!(get_Width, GetWidthFn)()
}

#[cfg(target_os = "windows")]
pub fn get_Width_orig() -> i32 {
    get_orig_fn!(get_Width, GetWidthFn)()
}

#[cfg(target_os = "windows")]
type GetHeightFn = extern "C" fn() -> i32;
#[cfg(target_os = "windows")]
extern "C" fn get_Height() -> i32 {
    if let Some((_, height)) = crate::windows::utils::get_scaling_res() {
        return height;
    }

    get_orig_fn!(get_Height, GetHeightFn)()
}

#[cfg(target_os = "windows")]
pub fn get_Height_orig() -> i32 {
    get_orig_fn!(get_Height, GetHeightFn)()
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, Screen);

    #[cfg(target_os = "android")]
    {
        let ChangeScreenOrientationLandscapeAsync_addr = get_method_addr(Screen, c"ChangeScreenOrientationLandscapeAsync", 0);
        let ChangeScreenOrientationPortraitAsync_addr = get_method_addr(Screen, c"ChangeScreenOrientationPortraitAsync", 0);

        new_hook!(ChangeScreenOrientationLandscapeAsync_addr, ChangeScreenOrientationLandscapeAsync);
        new_hook!(ChangeScreenOrientationPortraitAsync_addr, ChangeScreenOrientationPortraitAsync);
    }

    #[cfg(target_os = "windows")]
    {
        let get_Width_addr = get_method_addr(Screen, c"get_Width", 0);
        let get_Height_addr = get_method_addr(Screen, c"get_Height", 0);

        new_hook!(get_Width_addr, get_Width);
        new_hook!(get_Height_addr, get_Height);
    }
}