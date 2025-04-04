use mlua::{UserData, UserDataFields};
use libffi::middle::Type as FfiType;

use crate::il2cpp::{api::il2cpp_class_from_type, types::*, wrapper::Field};

use super::{Class, UnboundField};

wrapper_struct!(Type, *const Il2CppType);

impl Type {
    pub fn type_enum(&self) -> Il2CppTypeEnum {
        unsafe { (*self.0).type_() }
    }

    pub fn class(&self) -> Class {
        unsafe { Class::new_unchecked(il2cpp_class_from_type(self.0)) }
    }

    pub fn to_ffi_type(&self) -> Option<FfiType> {
        #[allow(non_upper_case_globals)]
        Some(match self.type_enum() {
            Il2CppTypeEnum_IL2CPP_TYPE_VOID => FfiType::void(),
            Il2CppTypeEnum_IL2CPP_TYPE_BOOLEAN => FfiType::i8(),
            Il2CppTypeEnum_IL2CPP_TYPE_CHAR => FfiType::u16(),
            Il2CppTypeEnum_IL2CPP_TYPE_I1 => FfiType::i8(),
            Il2CppTypeEnum_IL2CPP_TYPE_U1 => FfiType::u8(),
            Il2CppTypeEnum_IL2CPP_TYPE_I2 => FfiType::i16(),
            Il2CppTypeEnum_IL2CPP_TYPE_U2 => FfiType::u16(),
            Il2CppTypeEnum_IL2CPP_TYPE_I4 => FfiType::i32(),
            Il2CppTypeEnum_IL2CPP_TYPE_U4 => FfiType::u32(),
            Il2CppTypeEnum_IL2CPP_TYPE_I8 => FfiType::i64(),
            Il2CppTypeEnum_IL2CPP_TYPE_U8 => FfiType::u64(),
            Il2CppTypeEnum_IL2CPP_TYPE_R4 => FfiType::f32(),
            Il2CppTypeEnum_IL2CPP_TYPE_R8 => FfiType::f64(),
            Il2CppTypeEnum_IL2CPP_TYPE_I => FfiType::isize(),
            Il2CppTypeEnum_IL2CPP_TYPE_U => FfiType::usize(),
            Il2CppTypeEnum_IL2CPP_TYPE_VALUETYPE => {
                if self.class().is_enum() {
                    self.class().field(c"value__")?.type_().to_ffi_type()?
                }
                else {
                    let instance_fields: Vec<UnboundField> = self.class()
                        .fields()
                        .iter()
                        .filter(|f| !f.is_static())
                        .cloned()
                        .collect();
                    assert!(instance_fields.len() > 0);

                    let ffi_types = instance_fields.iter()
                        .map(|f| f.type_().to_ffi_type())
                        .collect::<Option<Vec<FfiType>>>()?;

                    FfiType::structure(ffi_types)
                }
            }
            Il2CppTypeEnum_IL2CPP_TYPE_STRING |
            Il2CppTypeEnum_IL2CPP_TYPE_PTR |
            Il2CppTypeEnum_IL2CPP_TYPE_BYREF |
            Il2CppTypeEnum_IL2CPP_TYPE_CLASS |
            Il2CppTypeEnum_IL2CPP_TYPE_ARRAY |
            Il2CppTypeEnum_IL2CPP_TYPE_GENERICINST |
            Il2CppTypeEnum_IL2CPP_TYPE_FNPTR |
            Il2CppTypeEnum_IL2CPP_TYPE_OBJECT |
            Il2CppTypeEnum_IL2CPP_TYPE_SZARRAY => FfiType::pointer(),
            _ => return None
        })
    }
}

impl UserData for Type {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        Self::add_raw_field(fields);
    }
}