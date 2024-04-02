use crate::il2cpp::types::*;

mod AnText;
pub mod AnMeshParameter;
pub mod AnMeshInfoParameterGroup;

pub fn init(image: *const Il2CppImage) {
    AnText::init(image);
    AnMeshParameter::init(image);
    AnMeshInfoParameterGroup::init(image);
}