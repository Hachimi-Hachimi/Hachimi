use crate::il2cpp::{api::il2cpp_resolve_icall, types::*};

static mut BLIT2_ADDR: usize = 0;
impl_addr_wrapper_fn!(Blit2, BLIT2_ADDR, (), source: *mut Il2CppObject, dest: *mut Il2CppObject);

pub fn init(_UnityEngine_CoreModule: *const Il2CppImage) {
    unsafe {
        BLIT2_ADDR = il2cpp_resolve_icall(
            c"UnityEngine.Graphics::Blit2(UnityEngine.Texture,UnityEngine.RenderTexture)".as_ptr()
        );
    }
}