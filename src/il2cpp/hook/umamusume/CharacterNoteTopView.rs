use crate::il2cpp::{symbols::get_method_addr, types::*};

static mut GET_BUTTONGALLERY_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_ButtonGallery, GET_BUTTONGALLERY_ADDR, *mut Il2CppObject, this: *mut Il2CppObject);

static mut GET_BUTTONTALKGALLERY_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_ButtonTalkGallery, GET_BUTTONTALKGALLERY_ADDR, *mut Il2CppObject, this: *mut Il2CppObject);

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, CharacterNoteTopView);

    unsafe {
        GET_BUTTONGALLERY_ADDR = get_method_addr(CharacterNoteTopView, c"get_ButtonGallery", 0);
        GET_BUTTONTALKGALLERY_ADDR = get_method_addr(CharacterNoteTopView, c"get_ButtonTalkGallery", 0);
    }
}