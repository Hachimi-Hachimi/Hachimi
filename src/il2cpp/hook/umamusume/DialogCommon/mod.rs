use crate::il2cpp::types::*;

pub mod Data;

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, DialogCommon);

    Data::init(DialogCommon)
}