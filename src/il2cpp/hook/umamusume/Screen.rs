use crate::{core::Hachimi, il2cpp::{symbols::{get_method_addr, IEnumerator, MoveNextFn}, types::*}};

use super::UIManager;

extern "C" fn ChangeScreenOrientationLandscapeAsync_MoveNext(enumerator: *mut Il2CppObject) -> bool {
    let moved = get_orig_fn!(ChangeScreenOrientationLandscapeAsync_MoveNext, MoveNextFn)(enumerator);
    if !moved {
        UIManager::apply_ui_scale();
    }
    moved
}

extern "C" fn ChangeScreenOrientationPortraitAsync_MoveNext(enumerator: *mut Il2CppObject) -> bool {
    let moved = get_orig_fn!(ChangeScreenOrientationPortraitAsync_MoveNext, MoveNextFn)(enumerator);
    if !moved {
        UIManager::apply_ui_scale();
    }
    moved
}

type ChangeScreenOrientationLandscapeAsyncFn = extern "C" fn() -> *mut Il2CppObject;
extern "C" fn ChangeScreenOrientationLandscapeAsync() -> *mut Il2CppObject {
    let res = get_orig_fn!(ChangeScreenOrientationLandscapeAsync, ChangeScreenOrientationLandscapeAsyncFn)();
    if Hachimi::instance().config.load().ui_scale == 1.0 { return res; }

    if let Some(enumerator) = <IEnumerator>::new(res) {
        if let Err(e) = enumerator.hook_move_next(ChangeScreenOrientationLandscapeAsync_MoveNext) {
            error!("Failed to hook enumerator: {}", e);
        }
    }
    res
}

type ChangeScreenOrientationPortraitAsyncFn = extern "C" fn() -> *mut Il2CppObject;
extern "C" fn ChangeScreenOrientationPortraitAsync() -> *mut Il2CppObject {
    let res = get_orig_fn!(ChangeScreenOrientationPortraitAsync, ChangeScreenOrientationPortraitAsyncFn)();
    if Hachimi::instance().config.load().ui_scale == 1.0 { return res; }

    if let Some(enumerator) = <IEnumerator>::new(res) {
        if let Err(e) = enumerator.hook_move_next(ChangeScreenOrientationPortraitAsync_MoveNext) {
            error!("Failed to hook enumerator: {}", e);
        }
    }
    res
}


pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, Screen);

    let ChangeScreenOrientationLandscapeAsync_addr = get_method_addr(Screen, c"ChangeScreenOrientationLandscapeAsync", 0);
    let ChangeScreenOrientationPortraitAsync_addr = get_method_addr(Screen, c"ChangeScreenOrientationPortraitAsync", 0);

    new_hook!(ChangeScreenOrientationLandscapeAsync_addr, ChangeScreenOrientationLandscapeAsync);
    new_hook!(ChangeScreenOrientationPortraitAsync_addr, ChangeScreenOrientationPortraitAsync);
}