use crate::il2cpp::{symbols::get_method_addr, types::*};

static mut GET_TARGETTEXT_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_TargetText, GET_TARGETTEXT_ADDR, *mut Il2CppObject, this: *mut Il2CppObject);

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, ButtonCommon);

    unsafe {
        GET_TARGETTEXT_ADDR = get_method_addr(ButtonCommon, c"get_TargetText", 0);
    }
}