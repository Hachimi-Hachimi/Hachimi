use crate::il2cpp::types::Il2CppImage;

pub mod SingleModeTurn;

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, MasterSingleModeTurn);

    SingleModeTurn::init(MasterSingleModeTurn);
}