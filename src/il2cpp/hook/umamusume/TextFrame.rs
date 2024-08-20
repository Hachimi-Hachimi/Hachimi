use std::{collections::hash_map, sync::Mutex};

use fnv::FnvHashMap;
use once_cell::sync::Lazy;

use crate::{core::Hachimi, il2cpp::{hook::UnityEngine_UI::Text, symbols::{get_method_addr, GCHandle}, types::*}};

static mut GET_TEXTLABEL_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_TextLabel, GET_TEXTLABEL_ADDR, *mut Il2CppObject, this: *mut Il2CppObject);

pub static PROCESSED: Lazy<Mutex<FnvHashMap<usize, GCHandle>>> = Lazy::new(|| Mutex::default());

type InitializeFn = extern "C" fn(this: *mut Il2CppObject);
extern "C" fn Initialize(this: *mut Il2CppObject) {
    get_orig_fn!(Initialize, InitializeFn)(this);

    if let hash_map::Entry::Vacant(e) = PROCESSED.lock().unwrap().entry(this as usize) {
        e.insert(GCHandle::new_weak_ref(this, false));
    }
    else {
        return;
    }

    let text_label = get_TextLabel(this);
    let localized_data = Hachimi::instance().localized_data.load();

    if let Some(mult) = localized_data.config.text_frame_line_spacing_multiplier {
        let line_spacing = Text::get_lineSpacing(text_label);
        Text::set_lineSpacing(text_label, line_spacing * mult);
    }
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, TextFrame);
    
    let Initialize_addr = get_method_addr(TextFrame, c"Initialize", 0);

    new_hook!(Initialize_addr, Initialize);

    unsafe {
        GET_TEXTLABEL_ADDR = get_method_addr(TextFrame, c"get_TextLabel", 0);
    }
}