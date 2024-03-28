use crate::{core::Hachimi, il2cpp::{symbols::get_method_addr, types::*}};

type GetCanvasSizeFn = extern "C" fn(this: *mut Il2CppObject) -> Vector2_t;
extern "C" fn GetCanvasSize(this: *mut Il2CppObject) -> Vector2_t {
    let mut size = get_orig_fn!(GetCanvasSize, GetCanvasSizeFn)(this);
    let mult = Hachimi::instance().config.load().virtual_res_mult;
    if mult != 1.0 {
        size.x /= mult;
        size.y /= mult;
    }
    size
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, CameraController);

    let GetCanvasSize_addr = get_method_addr(CameraController, cstr!("GetCanvasSize"), 0);

    new_hook!(GetCanvasSize_addr, GetCanvasSize);
}