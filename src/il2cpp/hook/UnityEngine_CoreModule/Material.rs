use crate::il2cpp::{symbols::get_method_addr, types::*};

static mut GET_MAINTEXTURE_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_mainTexture, GET_MAINTEXTURE_ADDR, *mut Il2CppObject, this: *mut Il2CppObject);

static mut SET_MAINTEXTURE_ADDR: usize = 0;
impl_addr_wrapper_fn!(set_mainTexture, SET_MAINTEXTURE_ADDR, (), this: *mut Il2CppObject, value: *mut Il2CppObject);

pub fn init(UnityEngine_CoreModule: *const Il2CppImage) {
    get_class_or_return!(UnityEngine_CoreModule, UnityEngine, Material);

    unsafe {
        GET_MAINTEXTURE_ADDR = get_method_addr(Material, c"get_mainTexture", 0);
        SET_MAINTEXTURE_ADDR = get_method_addr(Material, c"set_mainTexture", 1);
    }
}