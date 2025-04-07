macro_rules! wrapper_struct {
    ($name:ident, $ptr_type:ty) => {
        #[repr(transparent)]
        #[derive(Debug, Copy, Clone)]
        pub struct $name($ptr_type);

        impl $name {
            pub fn new(p: $ptr_type) -> Option<Self> {
                if p.is_null() { None } else { Some(Self(p)) }
            }

            pub unsafe fn new_unchecked(p: $ptr_type) -> Self {
                Self(p)
            }

            fn add_raw_field<F: mlua::UserDataFields<Self>>(fields: &mut F) {
                fields.add_field_method_get("raw", |_, v| Ok(crate::il2cpp::wrapper::NativePointer::Raw(v.0 as _)));
            }
        }

        impl crate::il2cpp::wrapper::GetRaw<$ptr_type> for $name {
            fn raw(&self) -> $ptr_type {
                self.0
            }
        }

        impl mlua::FromLua for $name {
            fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
                match value {
                    mlua::Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
                    _ => Err(mlua::Error::FromLuaConversionError {
                        from: value.type_name(),
                        to: stringify!($name).to_owned(),
                        message: None
                    })
                }
            }
        }
    };
}

pub trait GetRaw<T> {
    fn raw(&self) -> T;
}

mod domain;
mod assembly;
mod image;
mod class;
mod method;
mod field;
mod value;
mod bound_value;
mod object;
mod array;
mod string;
mod r#type;
mod value_type;
mod pointer;
mod reference;
mod exception;
mod native_pointer;

pub use domain::Domain;
pub use assembly::Assembly;
pub use image::Image;
pub use class::{Class, MethodFindMode};
pub use method::{Method, UnboundMethod, BoundMethod};
pub use field::{Field, FieldAttribute, UnboundField, BoundField};
pub use value::{Value, InvokerParam, FfiArg};
pub use bound_value::{BoundValue, BindableValue};
pub use object::Object;
pub use array::Array;
pub use string::String;
pub use r#type::Type;
pub use value_type::ValueType;
pub use pointer::Pointer;
pub use reference::Reference;
pub use exception::Exception;
pub use native_pointer::NativePointer;