use crate::il2cpp::types::*;

mod AnText;
pub mod AnMeshInfoParameterGroup;

pub fn init(image: *const Il2CppImage) {
    AnText::init(image);
    AnMeshInfoParameterGroup::init(image);
}