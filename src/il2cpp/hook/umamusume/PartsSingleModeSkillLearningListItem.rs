use crate::il2cpp::{symbols::get_method_addr, sql::TextDataQuery, types::*};

type UpdateCurrentFn = extern "C" fn(this: *mut Il2CppObject);
extern "C" fn UpdateCurrent(this: *mut Il2CppObject) {
    TextDataQuery::with_skill_learning_query(|| {
        get_orig_fn!(UpdateCurrent, UpdateCurrentFn)(this);
    });
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, PartsSingleModeSkillLearningListItem);

    let UpdateCurrent_addr = get_method_addr(PartsSingleModeSkillLearningListItem, c"UpdateCurrent", 0);

    new_hook!(UpdateCurrent_addr, UpdateCurrent);
}