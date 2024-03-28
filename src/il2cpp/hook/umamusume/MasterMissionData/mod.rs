use crate::il2cpp::types::*;

mod MissionData;

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, MasterMissionData);

    MissionData::init(MasterMissionData);
}