use crate::il2cpp::{symbols::get_method_addr, types::*};

static mut GET_LINESPACING_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_lineSpacing, GET_LINESPACING_ADDR, f32, this: *mut Il2CppObject);

static mut SET_LINESPACING_ADDR: usize = 0;
impl_addr_wrapper_fn!(set_lineSpacing, SET_LINESPACING_ADDR, (), this: *mut Il2CppObject, value: f32);

static mut GET_FONTSIZE_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_fontSize, GET_FONTSIZE_ADDR, i32, this: *mut Il2CppObject);

static mut SET_FONTSIZE_ADDR: usize = 0;
impl_addr_wrapper_fn!(set_fontSize, SET_FONTSIZE_ADDR, (), this: *mut Il2CppObject, value: i32);

pub fn init(UnityEngine_UI: *const Il2CppImage) {
    get_class_or_return!(UnityEngine_UI, "UnityEngine.UI", Text);
    
    unsafe {
        GET_LINESPACING_ADDR = get_method_addr(Text, c"get_lineSpacing", 0);
        SET_LINESPACING_ADDR = get_method_addr(Text, c"set_lineSpacing", 1);
        GET_FONTSIZE_ADDR = get_method_addr(Text, c"get_fontSize", 0);
        SET_FONTSIZE_ADDR = get_method_addr(Text, c"set_fontSize", 1);
    }
}