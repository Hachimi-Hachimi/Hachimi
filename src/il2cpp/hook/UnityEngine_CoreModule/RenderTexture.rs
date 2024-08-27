use crate::il2cpp::{api::il2cpp_resolve_icall, symbols::get_method_addr, types::*};

static mut GETTEMPORARY_ADDR: usize = 0;
impl_addr_wrapper_fn!(GetTemporary, GETTEMPORARY_ADDR, *mut Il2CppObject, width: i32, height: i32);

static mut RELEASETEMPORARY_ADDR: usize = 0;
impl_addr_wrapper_fn!(ReleaseTemporary, RELEASETEMPORARY_ADDR, (), temp: *mut Il2CppObject);

static mut GETACTIVE_ADDR: usize = 0;
impl_addr_wrapper_fn!(GetActive, GETACTIVE_ADDR, *mut Il2CppObject,);

static mut SETACTIVE_ADDR: usize = 0;
impl_addr_wrapper_fn!(SetActive, SETACTIVE_ADDR, (), value: *mut Il2CppObject);

pub fn init(UnityEngine_CoreModule: *const Il2CppImage) {
    get_class_or_return!(UnityEngine_CoreModule, UnityEngine, RenderTexture);

    unsafe {
        GETTEMPORARY_ADDR = get_method_addr(RenderTexture, c"GetTemporary", 2);
        RELEASETEMPORARY_ADDR = il2cpp_resolve_icall(
            c"UnityEngine.RenderTexture::ReleaseTemporary(UnityEngine.RenderTexture)".as_ptr()
        );
        GETACTIVE_ADDR = il2cpp_resolve_icall(
            c"UnityEngine.RenderTexture::GetActive()".as_ptr()
        );
        SETACTIVE_ADDR = il2cpp_resolve_icall(
            c"UnityEngine.RenderTexture::SetActive(UnityEngine.RenderTexture)".as_ptr()
        )
    }
}