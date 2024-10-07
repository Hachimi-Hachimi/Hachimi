use crate::il2cpp::{symbols::get_method_addr, types::*};

static mut GET_RACETYPE_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_RaceType, GET_RACETYPE_ADDR, i32, this: *mut Il2CppObject);

static mut SET_RACETYPE_ADDR: usize = 0;
impl_addr_wrapper_fn!(set_RaceType, SET_RACETYPE_ADDR, (), this: *mut Il2CppObject, value: i32);

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, RaceInfo);

    unsafe {
        GET_RACETYPE_ADDR = get_method_addr(RaceInfo, c"get_RaceType", 0);
        SET_RACETYPE_ADDR = get_method_addr(RaceInfo, c"set_RaceType", 1);
    }
}