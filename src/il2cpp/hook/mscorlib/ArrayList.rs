use crate::il2cpp::{symbols::get_method_addr, types::*};

static mut GET_ITEM_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_Item, GET_ITEM_ADDR, *mut Il2CppObject, this: *mut Il2CppObject, index: i32);

static mut SET_ITEM_ADDR: usize = 0;
impl_addr_wrapper_fn!(set_Item, SET_ITEM_ADDR, (), this: *mut Il2CppObject, index: i32, value: *mut Il2CppObject);

pub fn init(mscorlib: *const Il2CppImage) {
    get_class_or_return!(mscorlib, "System.Collections", ArrayList);

    unsafe {
        GET_ITEM_ADDR = get_method_addr(ArrayList, c"get_Item", -1);
        SET_ITEM_ADDR = get_method_addr(ArrayList, c"set_Item", -1);
    }
}