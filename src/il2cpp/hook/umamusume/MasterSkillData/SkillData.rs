use std::{ptr::null_mut, sync::atomic::{self, AtomicBool}};

use crate::{core::{ext::StringExt, utils, Hachimi}, il2cpp::{symbols::{get_field_from_name, get_field_value, get_method_addr}, types::*}};

static mut ID_FIELD: *mut FieldInfo = null_mut();
fn get_Id(this: *mut Il2CppObject) -> i32 {
    get_field_value(this, unsafe { ID_FIELD })
}

pub static IS_SKILL_LEARNING_QUERY: AtomicBool = AtomicBool::new(false);

// These values are guesstimated
const REMARKS_LINE_WIDTH: i32 = 18;
const REMARKS_LINE_COUNT: i32 = 4;
const REMARKS_FONT_SIZE: i32 = 28;

type GetRemarksFn = extern "C" fn(this: *mut Il2CppObject) -> *mut Il2CppString;
extern "C" fn get_Remarks(this: *mut Il2CppObject) -> *mut Il2CppString {
    // Do the text_data query replacement here directly since unique/inherited skills uses
    // a different id for some reason
    let mut id = get_Id(this);
    if id > 900000 {
        id -= 800000
    }
    let localized_data = Hachimi::instance().localized_data.load();
    let text_opt = localized_data
        .text_data_dict
        .get(&48)
        .map(|c| c.get(&id))
        .unwrap_or_default();
    let is_skill_learning_query = IS_SKILL_LEARNING_QUERY.compare_exchange(
        true,
        false,
        atomic::Ordering::Relaxed,
        atomic::Ordering::Relaxed
    ).is_ok();

    if let Some(text) = text_opt {
        // also because we need to do some prewrapping here
        // check PartsSingleModeSkillLearningListItem::Info to see when this value is set to true
        if is_skill_learning_query {
            if let Some(fitted) = utils::wrap_fit_text(text,
                REMARKS_LINE_WIDTH, REMARKS_LINE_COUNT, REMARKS_FONT_SIZE
            ) {
                return fitted.to_il2cpp_string();
            }
        }
        text.to_il2cpp_string()
    }
    else {
        get_orig_fn!(get_Remarks, GetRemarksFn)(this)
    }
}

pub fn init(MasterSkillData: *mut Il2CppClass) {
    find_nested_class_or_return!(MasterSkillData, SkillData);

    let get_Remarks_addr = get_method_addr(SkillData, cstr!("get_Remarks"), 0);

    new_hook!(get_Remarks_addr, get_Remarks);

    unsafe {
        ID_FIELD = get_field_from_name(SkillData, cstr!("Id"));
    }
}