use crate::il2cpp::types::*;

pub mod SkillData;

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, MasterSkillData);

    SkillData::init(MasterSkillData);
}