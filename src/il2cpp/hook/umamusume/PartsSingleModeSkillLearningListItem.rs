use crate::{core::sql::TextDataQuery, il2cpp::{symbols::get_method_addr, types::*}};

type UpdateSkillNameFn = extern "C" fn(this: *mut Il2CppObject);
extern "C" fn UpdateSkillName(this: *mut Il2CppObject) {
    TextDataQuery::with_skill_learning_query(|| {
        get_orig_fn!(UpdateSkillName, UpdateSkillNameFn)(this);
    });
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, PartsSingleModeSkillLearningListItem);

    let UpdateSkillName_addr = get_method_addr(PartsSingleModeSkillLearningListItem, c"UpdateSkillName", 0);

    new_hook!(UpdateSkillName_addr, UpdateSkillName);
}