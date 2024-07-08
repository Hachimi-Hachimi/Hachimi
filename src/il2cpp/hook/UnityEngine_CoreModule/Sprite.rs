use crate::il2cpp::{symbols::get_method_addr, types::*};

static mut GET_TEXTURE_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_texture, GET_TEXTURE_ADDR, *mut Il2CppObject, this: *mut Il2CppObject);

pub fn init(UnityEngine_CoreModule: *const Il2CppImage) {
    get_class_or_return!(UnityEngine_CoreModule, UnityEngine, Sprite);

    unsafe {
        GET_TEXTURE_ADDR = get_method_addr(Sprite, c"get_texture", 0);
    }
}