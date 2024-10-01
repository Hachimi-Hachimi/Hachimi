use crate::il2cpp::{hook::UnityEngine_TextRenderingModule::TextAnchor, symbols::get_method_addr, types::*};

static mut GET_LINESPACING_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_lineSpacing, GET_LINESPACING_ADDR, f32, this: *mut Il2CppObject);

static mut SET_LINESPACING_ADDR: usize = 0;
impl_addr_wrapper_fn!(set_lineSpacing, SET_LINESPACING_ADDR, (), this: *mut Il2CppObject, value: f32);

static mut GET_FONTSIZE_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_fontSize, GET_FONTSIZE_ADDR, i32, this: *mut Il2CppObject);

static mut SET_FONTSIZE_ADDR: usize = 0;
impl_addr_wrapper_fn!(set_fontSize, SET_FONTSIZE_ADDR, (), this: *mut Il2CppObject, value: i32);

static mut SET_FONT_ADDR: usize = 0;
impl_addr_wrapper_fn!(set_font, SET_FONT_ADDR, (), this: *mut Il2CppObject, value: *mut Il2CppObject);

static mut SET_HORIZONTALOVERFLOW_ADDR: usize = 0;
impl_addr_wrapper_fn!(set_horizontalOverflow, SET_HORIZONTALOVERFLOW_ADDR, (), this: *mut Il2CppObject, value: i32);

static mut SET_VERTICALOVERFLOW_ADDR: usize = 0;
impl_addr_wrapper_fn!(set_verticalOverflow, SET_VERTICALOVERFLOW_ADDR, (), this: *mut Il2CppObject, value: i32);

static mut GET_TEXT_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_text, GET_TEXT_ADDR, *mut Il2CppString, this: *mut Il2CppObject);

static mut SET_TEXT_ADDR: usize = 0;
impl_addr_wrapper_fn!(set_text, SET_TEXT_ADDR, (), this: *mut Il2CppObject, value: *mut Il2CppString);

static mut SET_ALIGNMENT_ADDR: usize = 0;
impl_addr_wrapper_fn!(set_alignment, SET_ALIGNMENT_ADDR, (), this: *mut Il2CppObject, value: TextAnchor);

pub fn init(UnityEngine_UI: *const Il2CppImage) {
    get_class_or_return!(UnityEngine_UI, "UnityEngine.UI", Text);
    
    unsafe {
        GET_LINESPACING_ADDR = get_method_addr(Text, c"get_lineSpacing", 0);
        SET_LINESPACING_ADDR = get_method_addr(Text, c"set_lineSpacing", 1);
        GET_FONTSIZE_ADDR = get_method_addr(Text, c"get_fontSize", 0);
        SET_FONTSIZE_ADDR = get_method_addr(Text, c"set_fontSize", 1);
        SET_FONT_ADDR = get_method_addr(Text, c"set_font", 1);
        SET_HORIZONTALOVERFLOW_ADDR = get_method_addr(Text, c"set_horizontalOverflow", 1);
        SET_VERTICALOVERFLOW_ADDR = get_method_addr(Text, c"set_verticalOverflow", 1);
        GET_TEXT_ADDR = get_method_addr(Text, c"get_text", 0);
        SET_TEXT_ADDR = get_method_addr(Text, c"set_text", 1);
        SET_ALIGNMENT_ADDR = get_method_addr(Text, c"set_alignment", 1);
    }
}