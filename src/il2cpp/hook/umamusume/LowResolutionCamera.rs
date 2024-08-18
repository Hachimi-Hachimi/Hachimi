use std::sync::atomic::{self, AtomicBool};

use crate::il2cpp::{symbols::get_method_addr, types::*};

static CREATING_RENDER_TEXTURE: AtomicBool = AtomicBool::new(false);
pub fn creating_render_texture() -> bool {
    CREATING_RENDER_TEXTURE.load(atomic::Ordering::Relaxed)
}

fn with_creating_render_texture(callback: impl FnOnce()) {
    CREATING_RENDER_TEXTURE.store(true, atomic::Ordering::Relaxed);
    callback();
    CREATING_RENDER_TEXTURE.store(false, atomic::Ordering::Relaxed);
}

type InitializeFn = extern "C" fn(this: *mut Il2CppObject, bg_path: *mut Il2CppObject);
extern "C" fn Initialize(this: *mut Il2CppObject, view: *mut Il2CppObject) {
    with_creating_render_texture(|| {
        get_orig_fn!(Initialize, InitializeFn)(this, view);
    });
}

type RemakeRendererTextureFn = extern "C" fn(this: *mut Il2CppObject);
extern "C" fn RemakeRendererTexture(this: *mut Il2CppObject) {
    with_creating_render_texture(|| {
        get_orig_fn!(RemakeRendererTexture, RemakeRendererTextureFn)(this);
    });
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, LowResolutionCamera);

    let Initialize_addr = get_method_addr(LowResolutionCamera, c"Initialize", 1);
    let RemakeRendererTexture_addr = get_method_addr(LowResolutionCamera, c"RemakeRendererTexture", 0);

    new_hook!(Initialize_addr, Initialize);
    new_hook!(RemakeRendererTexture_addr, RemakeRendererTexture);
}