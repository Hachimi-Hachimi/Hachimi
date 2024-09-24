use crate::il2cpp::{symbols::get_method_addr, types::*};

static mut CREATEFONTASSET_ADDR: usize = 0;
impl_addr_wrapper_fn!(CreateFontAsset, CREATEFONTASSET_ADDR, *mut Il2CppObject, font: *mut Il2CppObject);

pub fn init(Unity_TextMeshPro: *const Il2CppImage) {
    get_class_or_return!(Unity_TextMeshPro, TMPro, TMP_FontAsset);

    unsafe {
        CREATEFONTASSET_ADDR = get_method_addr(TMP_FontAsset, c"CreateFontAsset", 1);
    }
}