use crate::il2cpp::{symbols::get_method_addr, types::*};

static mut SET_FONT_ADDR: usize = 0;
impl_addr_wrapper_fn!(set_font, SET_FONT_ADDR, (), this: *mut Il2CppObject, value: *mut Il2CppObject);

pub fn init(Unity_TextMeshPro: *const Il2CppImage) {
    get_class_or_return!(Unity_TextMeshPro, TMPro, TMP_Text);

    unsafe {
        SET_FONT_ADDR = get_method_addr(TMP_Text, c"set_font", 1);
    }
}