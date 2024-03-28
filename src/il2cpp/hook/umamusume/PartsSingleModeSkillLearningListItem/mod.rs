use crate::il2cpp::types::*;

mod Info;

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, PartsSingleModeSkillLearningListItem);

    Info::init(PartsSingleModeSkillLearningListItem);
}