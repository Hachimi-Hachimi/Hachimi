use crate::{core::utils, il2cpp::{symbols::get_method_addr, types::*}};

// These values are guesstimated
const NAME_LINE_WIDTH: i32 = 16;
const NAME_LINE_COUNT: i32 = 2;
const NAME_FONT_SIZE: i32 = 36;

type GetNameFn = extern "C" fn(this: *mut Il2CppObject) -> *mut Il2CppString;
extern "C" fn get_Name(this: *mut Il2CppObject) -> *mut Il2CppString {
    let text = get_orig_fn!(get_Name, GetNameFn)(this);
    if let Some(fitted) = utils::wrap_fit_text_il2cpp(text, NAME_LINE_WIDTH, NAME_LINE_COUNT, NAME_FONT_SIZE) {
        fitted
    }
    else {
        text
    }
}

pub fn init(MasterMissionData: *mut Il2CppClass) {
    find_nested_class_or_return!(MasterMissionData, MissionData);

    let get_Name_addr = get_method_addr(MissionData, c"get_Name", 0);

    new_hook!(get_Name_addr, get_Name);
}