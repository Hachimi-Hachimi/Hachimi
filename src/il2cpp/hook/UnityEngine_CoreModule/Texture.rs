use crate::il2cpp::{api::il2cpp_resolve_icall, types::*};

static mut GETDATAWIDTH_ADDR: usize = 0;
impl_addr_wrapper_fn!(GetDataWidth, GETDATAWIDTH_ADDR, i32, this: *mut Il2CppObject);

static mut GETDATAHEIGHT_ADDR: usize = 0;
impl_addr_wrapper_fn!(GetDataHeight, GETDATAHEIGHT_ADDR, i32, this: *mut Il2CppObject);

pub fn init(_UnityEngine_CoreModule: *const Il2CppImage) {
    unsafe {
        GETDATAWIDTH_ADDR = il2cpp_resolve_icall(c"UnityEngine.Texture::GetDataWidth()".as_ptr());
        GETDATAHEIGHT_ADDR = il2cpp_resolve_icall(c"UnityEngine.Texture::GetDataHeight()".as_ptr());
    }
}