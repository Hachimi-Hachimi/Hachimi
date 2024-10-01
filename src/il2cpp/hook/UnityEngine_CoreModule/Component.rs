use crate::il2cpp::{api::il2cpp_resolve_icall, types::*};

static mut GET_GAMEOBJECT_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_gameObject, GET_GAMEOBJECT_ADDR, *mut Il2CppObject, this: *mut Il2CppObject);

pub fn init(_UnityEngine_CoreModule: *const Il2CppImage) {
    unsafe {
        GET_GAMEOBJECT_ADDR = il2cpp_resolve_icall(c"UnityEngine.Component::get_gameObject()".as_ptr());
    }
}