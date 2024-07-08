use std::ptr::null_mut;

use crate::il2cpp::{api::{il2cpp_object_new, il2cpp_runtime_object_init}, symbols::get_method_addr, types::*};

static mut CLASS: *mut Il2CppClass = null_mut();
pub fn class() -> *mut Il2CppClass {
    unsafe { CLASS }
}

pub fn new() -> *mut Il2CppObject {
    let object = il2cpp_object_new(class());
    il2cpp_runtime_object_init(object);
    object
}

static mut SETSIMPLEONEBUTTONMESSAGE_ADDR: usize = 0;
impl_addr_wrapper_fn!(SetSimpleOneButtonMessage, SETSIMPLEONEBUTTONMESSAGE_ADDR, *mut Il2CppObject,
    this: *mut Il2CppObject, header_text: *mut Il2CppString,
    message: *mut Il2CppString, on_click_center_button: *mut Il2CppDelegate,
    close_text_id: i32, dialog_form_type: i32
);

pub fn init(DialogCommon: *mut Il2CppClass) {
    find_nested_class_or_return!(DialogCommon, Data);

    unsafe {
        CLASS = Data;
        SETSIMPLEONEBUTTONMESSAGE_ADDR = get_method_addr(Data, c"SetSimpleOneButtonMessage", 5);
    }
}