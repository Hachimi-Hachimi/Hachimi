use crate::il2cpp::{symbols::get_method_addr, types::*};

static mut GETVIEW_ADDR: usize = 0;
impl_addr_wrapper_fn!(GetView, GETVIEW_ADDR, *mut Il2CppObject, this: *mut Il2CppObject);

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, CharacterNoteTopViewController);

    unsafe {
        GETVIEW_ADDR = get_method_addr(CharacterNoteTopViewController, c"GetView", 0);
    }
}