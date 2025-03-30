use mlua::{UserData, UserDataFields};

use crate::il2cpp::{api::*, types::*};

use super::Image;

wrapper_struct!(Assembly, *const Il2CppAssembly);

impl Assembly {
    pub fn image(&self) -> Option<Image> {
        Image::new(il2cpp_assembly_get_image(self.0))
    }
}

impl UserData for Assembly {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        Self::add_raw_field(fields);
        fields.add_field_method_get("image", |_, assembly| Ok(assembly.image()));
    }
}