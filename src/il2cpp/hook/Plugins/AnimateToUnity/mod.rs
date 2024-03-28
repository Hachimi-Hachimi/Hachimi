use crate::il2cpp::types::*;

mod AnText;

pub fn init(image: *const Il2CppImage) {
    AnText::init(image);
}