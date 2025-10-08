use std::sync::atomic::{self, AtomicBool};

use crate::{il2cpp::{symbols::get_method_addr, types::*}, windows::wnd_hook};

static SPLASH_SHOWN: AtomicBool = AtomicBool::new(false);
pub fn is_splash_shown() -> bool {
    SPLASH_SHOWN.load(atomic::Ordering::Acquire)
}

type ChangeViewFn = extern "C" fn(
    this: *mut Il2CppObject, next_view_id: i32, view_info: *mut Il2CppObject,
    callback_on_change_view_cancel: *mut Il2CppObject, callback_on_change_view_accept: *mut Il2CppObject,
    force_change: bool
);
extern "C" fn ChangeView(
    this: *mut Il2CppObject, next_view_id: i32, view_info: *mut Il2CppObject,
    callback_on_change_view_cancel: *mut Il2CppObject, callback_on_change_view_accept: *mut Il2CppObject,
    force_change: bool
) {
    get_orig_fn!(ChangeView, ChangeViewFn)(
        this, next_view_id, view_info, callback_on_change_view_cancel, callback_on_change_view_accept, force_change
    );
    if next_view_id == 1 { // ViewId.Splash
        SPLASH_SHOWN.store(true, atomic::Ordering::Release);
    }
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, SceneManager);

    let ChangeView_addr = get_method_addr(SceneManager, c"ChangeView", 5);

    new_hook!(ChangeView_addr, ChangeView);
}