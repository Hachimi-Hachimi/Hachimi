use std::sync::atomic;

use crate::{core::utils, il2cpp::{hook::umamusume::MasterSkillData, symbols::get_method_addr, types::*}};

// These values are guesstimated
const NAME_LINE_WIDTH: i32 = 13;
const NAME_FONT_SIZE: i32 = 32;

type GetNameFn = extern "C" fn(this: *mut Il2CppObject) -> *mut Il2CppString;
extern "C" fn get_Name(this: *mut Il2CppObject) -> *mut Il2CppString {
    // HEURISTIC: we assume that right after a name query, the item will also do a remarks query
    // because there's no class member specific to the remarks that we can hook into directly
    MasterSkillData::SkillData::IS_SKILL_LEARNING_QUERY.store(true, atomic::Ordering::Relaxed);

    let text = get_orig_fn!(get_Name, GetNameFn)(this);
    if let Some(fitted) = utils::fit_text_il2cpp(text, NAME_LINE_WIDTH, NAME_FONT_SIZE) {
        fitted
    }
    else {
        text
    }
}

pub fn init(PartsSingleModeSkillLearningListItem: *mut Il2CppClass) {
    find_nested_class_or_return!(PartsSingleModeSkillLearningListItem, Info);

    let get_Name_addr = get_method_addr(Info, cstr!("get_Name"), 0);

    new_hook!(get_Name_addr, get_Name);
}