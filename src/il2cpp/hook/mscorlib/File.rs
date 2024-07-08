use crate::il2cpp::{symbols::get_method_addr, types::*};

static mut READALLBYTES_ADDR: usize = 0;
impl_addr_wrapper_fn!(ReadAllBytes, READALLBYTES_ADDR, *mut Il2CppArray, path: *const Il2CppString);

pub fn init(mscorlib: *const Il2CppImage) {
    get_class_or_return!(mscorlib, "System.IO", File);

    unsafe {
        READALLBYTES_ADDR = get_method_addr(File, c"ReadAllBytes", 1);
    }
}