use std::{marker::PhantomData, os::raw::c_void};

use mlua::IntoLua;

use crate::il2cpp::{types::*, wrapper::GetRaw};

use super::{Array, Object, Pointer, Reference, Type, ValueType};

#[derive(Clone)]
pub enum Value {
    Void,
    Boolean(bool),
    Char(u16),
    I1(i8),
    U1(u8),
    I2(i16),
    U2(u16),
    I4(i32),
    U4(u32),
    I8(i64),
    U8(u64),
    R4(f32),
    R8(f64),
    String(super::String),
    Pointer(Pointer),
    Reference(Reference),
    ValueType(ValueType),
    Class(Object),
    Array(Array),
    GenericInstance(Object),
    Object(Object),
    SzArray(Array)
}

impl Value {
    /*
    pub fn number(&self) -> Option<f64> {
        Some(match self {
            Self::I1(v) => *v as _,
            Self::U1(v) => *v as _,
            Self::I2(v) => *v as _,
            Self::U2(v) => *v as _,
            Self::I4(v) => *v as _,
            Self::U4(v) => *v as _,
            Self::I8(v) => *v as _,
            Self::U8(v) => *v as _,
            Self::R4(v) => *v as _,
            Self::R8(v) => *v,
            _ => return None
        })
    }
    */

    pub const NULL: Value = Value::Void;

    pub fn is_void(&self) -> bool {
        match self {
            Self::Void => true,
            _ => false
        }
    }

    pub fn is_null(&self) -> bool {
        self.is_void()
    }

    #[allow(non_upper_case_globals)]
    pub unsafe fn from_invoker_return(value: *mut Il2CppObject, type_: Type) -> Value {
        let Some(object) = Object::new(value) else {
            return Value::NULL;
        };

        // TODO: handle valuetype
        match type_.type_enum() {
            Il2CppTypeEnum_IL2CPP_TYPE_OBJECT => Self::Object(object),
            Il2CppTypeEnum_IL2CPP_TYPE_CLASS => Self::Class(object),
            Il2CppTypeEnum_IL2CPP_TYPE_GENERICINST => Self::GenericInstance(object),
            Il2CppTypeEnum_IL2CPP_TYPE_STRING => Self::String(object.as_string()),
            _ => todo!()
        }
    }

    #[allow(non_upper_case_globals)]
    unsafe fn to_invoker_param_raw(&self, type_: Type) -> Option<*const c_void> {
        use std::ptr::from_ref;

        let type_enum = type_.type_enum();
        let class = type_.class();

        match self {
            Value::Void => return Some(0 as _), // All value types are nullable
            Value::Boolean(v) => if type_enum == Il2CppTypeEnum_IL2CPP_TYPE_BOOLEAN {
                return Some(from_ref(v) as _)
            },
            Value::Char(v) => match type_enum {
                Il2CppTypeEnum_IL2CPP_TYPE_CHAR => return Some(from_ref(v) as _),
                Il2CppTypeEnum_IL2CPP_TYPE_VALUETYPE => if class.is_enum_with_type(Il2CppTypeEnum_IL2CPP_TYPE_CHAR) {
                    return Some(from_ref(v) as _)
                },
                _ => ()
            },
            Value::I1(v) => match type_enum {
                Il2CppTypeEnum_IL2CPP_TYPE_I1 => return Some(from_ref(v) as _),
                Il2CppTypeEnum_IL2CPP_TYPE_VALUETYPE => if class.is_enum_with_type(Il2CppTypeEnum_IL2CPP_TYPE_I1) {
                    return Some(from_ref(v) as _)
                },
                _ => ()
            },
            Value::U1(v) => match type_enum {
                Il2CppTypeEnum_IL2CPP_TYPE_U1 => return Some(from_ref(v) as _),
                Il2CppTypeEnum_IL2CPP_TYPE_VALUETYPE => if class.is_enum_with_type(Il2CppTypeEnum_IL2CPP_TYPE_U1) {
                    return Some(from_ref(v) as _)
                },
                _ => ()
            },
            Value::I2(v) => match type_enum {
                Il2CppTypeEnum_IL2CPP_TYPE_I2 => return Some(from_ref(v) as _),
                Il2CppTypeEnum_IL2CPP_TYPE_VALUETYPE => if class.is_enum_with_type(Il2CppTypeEnum_IL2CPP_TYPE_I2) {
                    return Some(from_ref(v) as _)
                },
                _ => ()
            },
            Value::U2(v) => match type_enum {
                Il2CppTypeEnum_IL2CPP_TYPE_U2 => return Some(from_ref(v) as _),
                Il2CppTypeEnum_IL2CPP_TYPE_VALUETYPE => if class.is_enum_with_type(Il2CppTypeEnum_IL2CPP_TYPE_U2) {
                    return Some(from_ref(v) as _)
                },
                _ => ()
            },
            Value::I4(v) => match type_enum {
                Il2CppTypeEnum_IL2CPP_TYPE_I4 => return Some(from_ref(v) as _),
                Il2CppTypeEnum_IL2CPP_TYPE_VALUETYPE => if class.is_enum_with_type(Il2CppTypeEnum_IL2CPP_TYPE_I4) {
                    return Some(from_ref(v) as _)
                },
                _ => ()
            },
            Value::U4(v) => match type_enum {
                Il2CppTypeEnum_IL2CPP_TYPE_U4 => return Some(from_ref(v) as _),
                Il2CppTypeEnum_IL2CPP_TYPE_VALUETYPE => if class.is_enum_with_type(Il2CppTypeEnum_IL2CPP_TYPE_U4) {
                    return Some(from_ref(v) as _)
                },
                _ => ()
            },
            Value::I8(v) => match type_enum {
                Il2CppTypeEnum_IL2CPP_TYPE_I8 => return Some(from_ref(v) as _),
                Il2CppTypeEnum_IL2CPP_TYPE_VALUETYPE => if class.is_enum_with_type(Il2CppTypeEnum_IL2CPP_TYPE_I8) {
                    return Some(from_ref(v) as _)
                },
                _ => ()
            },
            Value::U8(v) => match type_enum {
                Il2CppTypeEnum_IL2CPP_TYPE_U8 => return Some(from_ref(v) as _),
                Il2CppTypeEnum_IL2CPP_TYPE_VALUETYPE => if class.is_enum_with_type(Il2CppTypeEnum_IL2CPP_TYPE_U8) {
                    return Some(from_ref(v) as _)
                },
                _ => ()
            },
            Value::R4(v) => if type_enum == Il2CppTypeEnum_IL2CPP_TYPE_R4 {
                return Some(from_ref(v) as _)
            },
            Value::R8(v) => if type_enum == Il2CppTypeEnum_IL2CPP_TYPE_R8 {
                return Some(from_ref(v) as _)
            },
            Value::String(string) => if type_enum == Il2CppTypeEnum_IL2CPP_TYPE_STRING {
                return Some(string.raw() as _)
            },
            Value::Pointer(pointer) => match type_enum {
                Il2CppTypeEnum_IL2CPP_TYPE_PTR |
                Il2CppTypeEnum_IL2CPP_TYPE_FNPTR => return Some(pointer.ptr() as _),
                _ => ()
            },
            Value::Reference(reference) => if type_enum == Il2CppTypeEnum_IL2CPP_TYPE_BYREF {
                return Some(reference.ptr() as _)
            },
            Value::ValueType(value_type) => if type_enum == Il2CppTypeEnum_IL2CPP_TYPE_VALUETYPE {
                return Some(value_type.ptr().get() as _)
            },
            Value::Object(object) | Value::Class(object) | Value::GenericInstance(object) => match type_enum {
                Il2CppTypeEnum_IL2CPP_TYPE_OBJECT |
                Il2CppTypeEnum_IL2CPP_TYPE_CLASS |
                Il2CppTypeEnum_IL2CPP_TYPE_GENERICINST => return Some(object.raw() as _),
                _ => (),
            },
            Value::Array(array) | Value::SzArray(array) => match type_enum {
                Il2CppTypeEnum_IL2CPP_TYPE_ARRAY |
                Il2CppTypeEnum_IL2CPP_TYPE_SZARRAY => return Some(array.raw() as _),
                _ => (),
            }
        }

        None
    }

    pub unsafe fn to_invoker_param(&self, type_: Type) -> Option<InvokerParam> {
        self.to_invoker_param_raw(type_).map(|ptr| InvokerParam::new(ptr, self))
    }
}

impl IntoLua for Value {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        Ok(match self {
            Value::Void => mlua::Value::Nil,
            Value::Boolean(b) => mlua::Value::Boolean(b),
            Value::Char(c) => mlua::Value::Integer(c as _),
            Value::I1(i) => mlua::Value::Integer(i as _),
            Value::U1(u) => mlua::Value::Integer(u as _),
            Value::I2(i) => mlua::Value::Integer(i as _),
            Value::U2(u) => mlua::Value::Integer(u as _),
            Value::I4(i) => mlua::Value::Integer(i as _),
            Value::U4(u) => mlua::Value::Integer(u as _),
            Value::I8(i) => mlua::Value::Integer(i),
            Value::U8(u) => mlua::Value::Integer(u as _),
            Value::R4(f) => mlua::Value::Number(f as _),
            Value::R8(f) => mlua::Value::Number(f),
            Value::String(s) => mlua::Value::UserData(lua.create_userdata(s)?),
            Value::Pointer(p) => mlua::Value::UserData(lua.create_userdata(p)?),
            Value::Reference(r) => mlua::Value::UserData(lua.create_userdata(r)?),
            Value::ValueType(v) => mlua::Value::UserData(lua.create_userdata(v)?),
            Value::Class(o) => mlua::Value::UserData(lua.create_userdata(o)?),
            Value::Array(a) => mlua::Value::UserData(lua.create_userdata(a)?),
            Value::GenericInstance(o) => mlua::Value::UserData(lua.create_userdata(o)?),
            Value::Object(o) => mlua::Value::UserData(lua.create_userdata(o)?),
            Value::SzArray(a) => mlua::Value::UserData(lua.create_userdata(a)?),
        })
    }
}

impl From<()> for Value {
    fn from(_value: ()) -> Self {
        Self::Void
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<i8> for Value {
    fn from(value: i8) -> Self {
        Self::I1(value)
    }
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Self::U1(value)
    }
}

impl From<i16> for Value {
    fn from(value: i16) -> Self {
        Self::I2(value)
    }
}

impl From<u16> for Value {
    fn from(value: u16) -> Self {
        Self::U2(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self::I4(value)
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Self::U4(value)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::I8(value)
    }
}

impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Self::U8(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Self::R4(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::R8(value)
    }
}

impl From<super::String> for Value {
    fn from(value: super::String) -> Self {
        Self::String(value)
    }
}

impl From<Pointer> for Value {
    fn from(value: Pointer) -> Self {
        Self::Pointer(value)
    }
}

impl From<Reference> for Value {
    fn from(value: Reference) -> Self {
        Self::Reference(value)
    }
}

impl From<ValueType> for Value {
    fn from(value: ValueType) -> Self {
        Self::ValueType(value)
    }
}

impl From<Array> for Value {
    fn from(value: Array) -> Self {
        Self::Array(value)
    }
}

impl From<Object> for Value {
    fn from(value: Object) -> Self {
        Self::Object(value)
    }
}

#[repr(transparent)]
pub struct InvokerParam<'a> {
    ptr: *const c_void,
    _lifetime: PhantomData<&'a Value>
}

impl<'a> InvokerParam<'a> {
    fn new(ptr: *const c_void, _value: &'a Value) -> InvokerParam {
        InvokerParam {
            ptr,
            _lifetime: PhantomData
        }
    }

    pub fn get(&self) -> *const c_void {
        self.ptr
    }
}