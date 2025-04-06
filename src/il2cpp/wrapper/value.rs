use std::{marker::PhantomData, os::raw::c_void};

use mlua::IntoLua;

use crate::il2cpp::{types::*, wrapper::GetRaw, Error};

use super::{Array, Field, Object, Pointer, Reference, Type, ValueType};

#[derive(Debug, Clone)]
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
    I(isize),
    U(usize),
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
            Value::I(v) => match type_enum {
                Il2CppTypeEnum_IL2CPP_TYPE_I => return Some(from_ref(v) as _),
                Il2CppTypeEnum_IL2CPP_TYPE_VALUETYPE => if class.is_enum_with_type(Il2CppTypeEnum_IL2CPP_TYPE_I) {
                    return Some(from_ref(v) as _)
                },
                _ => ()
            },
            Value::U(v) => match type_enum {
                Il2CppTypeEnum_IL2CPP_TYPE_U => return Some(from_ref(v) as _),
                Il2CppTypeEnum_IL2CPP_TYPE_VALUETYPE => if class.is_enum_with_type(Il2CppTypeEnum_IL2CPP_TYPE_U) {
                    return Some(from_ref(v) as _)
                },
                _ => ()
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

    pub unsafe fn to_ffi_arg(&self, type_: Type) -> Option<FfiArg> {
        let type_enum = type_.type_enum();

        #[allow(non_upper_case_globals)]
        match self {
            // these types are passed by reference or a raw pointer value
            Value::String(string) => if type_enum == Il2CppTypeEnum_IL2CPP_TYPE_STRING {
                return Some(FfiArg::new(string as *const _ as _, self))
            },
            Value::Pointer(pointer) => match type_enum {
                Il2CppTypeEnum_IL2CPP_TYPE_PTR |
                Il2CppTypeEnum_IL2CPP_TYPE_FNPTR =>
                    return Some(FfiArg::new(pointer as *const _ as _, self)),
                _ => ()
            },
            Value::Reference(reference) => if type_enum == Il2CppTypeEnum_IL2CPP_TYPE_BYREF {
                return Some(FfiArg::new(reference as *const _ as _, self))
            },
            Value::Object(object) | Value::Class(object) | Value::GenericInstance(object) => match type_enum {
                Il2CppTypeEnum_IL2CPP_TYPE_OBJECT |
                Il2CppTypeEnum_IL2CPP_TYPE_CLASS |
                Il2CppTypeEnum_IL2CPP_TYPE_GENERICINST =>
                    return Some(FfiArg::new(object as *const _ as _, self)),
                _ => (),
            },
            Value::Array(array) | Value::SzArray(array) => match type_enum {
                Il2CppTypeEnum_IL2CPP_TYPE_ARRAY |
                Il2CppTypeEnum_IL2CPP_TYPE_SZARRAY =>
                    return Some(FfiArg::new(array as *const _ as _, self)),
                _ => (),
            }
            // all other types has the same representation as invoker param
            _ => return self.to_invoker_param_raw(type_).map(|ptr| FfiArg::new(ptr, self))
        }

        None
    }

    pub fn from_lua(value: &mlua::Value, type_: Type) -> Option<Self> {
        // All value types are allowed to be null
        if value.is_nil() {
            return Some(Value::NULL);
        }
        #[allow(non_upper_case_globals)]
        match type_.type_enum() {
            // expect null value but not nil (checked above)
            Il2CppTypeEnum_IL2CPP_TYPE_VOID => (),
            Il2CppTypeEnum_IL2CPP_TYPE_BOOLEAN => {
                if let mlua::Value::Boolean(b) = value {
                    return Some(Value::Boolean(*b))
                }
            }
            Il2CppTypeEnum_IL2CPP_TYPE_CHAR => {
                if let mlua::Value::Integer(i) = value {
                    return Some(Value::Char((*i).try_into().ok()?))
                }
            }
            Il2CppTypeEnum_IL2CPP_TYPE_I1 => {
                if let mlua::Value::Integer(i) = value {
                    return Some(Value::I1((*i).try_into().ok()?))
                }
            }
            Il2CppTypeEnum_IL2CPP_TYPE_U1 => {
                if let mlua::Value::Integer(i) = value {
                    return Some(Value::U1((*i).try_into().ok()?))
                }
            }
            Il2CppTypeEnum_IL2CPP_TYPE_I2 => {
                if let mlua::Value::Integer(i) = value {
                    return Some(Value::I2((*i).try_into().ok()?))
                }
            }
            Il2CppTypeEnum_IL2CPP_TYPE_U2 => {
                if let mlua::Value::Integer(i) = value {
                    return Some(Value::U2((*i).try_into().ok()?))
                }
            }
            Il2CppTypeEnum_IL2CPP_TYPE_I4 => {
                if let mlua::Value::Integer(i) = value {
                    return Some(Value::I4((*i).try_into().ok()?))
                }
            }
            Il2CppTypeEnum_IL2CPP_TYPE_U4 => {
                if let mlua::Value::Integer(i) = value {
                    return Some(Value::U4((*i).try_into().ok()?))
                }
            }
            Il2CppTypeEnum_IL2CPP_TYPE_I8 => {
                if let mlua::Value::Integer(i) = value {
                    return Some(Value::I8(*i))
                }
            }
            Il2CppTypeEnum_IL2CPP_TYPE_U8 => {
                if let mlua::Value::Integer(i) = value {
                    return Some(Value::U8(*i as u64))
                }
            }
            Il2CppTypeEnum_IL2CPP_TYPE_R4 => {
                if let mlua::Value::Number(v) = value {
                    return Some(Value::R4(*v as f32))
                }
            }
            Il2CppTypeEnum_IL2CPP_TYPE_R8 => {
                if let mlua::Value::Number(v) = value {
                    return Some(Value::R8(*v))
                }
            }
            Il2CppTypeEnum_IL2CPP_TYPE_STRING => {
                match value {
                    mlua::Value::UserData(ud) => return Some(Value::String(*ud.borrow::<super::String>().ok()?)),
                    mlua::Value::String(s) => return Some(Value::String(s.to_string_lossy().into())),
                    _ => ()
                }
            }
            Il2CppTypeEnum_IL2CPP_TYPE_PTR | Il2CppTypeEnum_IL2CPP_TYPE_FNPTR => {
                if let mlua::Value::UserData(ud) = value {
                    return Some(Value::Pointer(*ud.borrow::<Pointer>().ok()?))
                }
            }
            Il2CppTypeEnum_IL2CPP_TYPE_BYREF => {
                if let mlua::Value::UserData(ud) = value {
                    return Some(Value::Reference(*ud.borrow::<Reference>().ok()?))
                }
            }
            Il2CppTypeEnum_IL2CPP_TYPE_VALUETYPE => {
                match value {
                    mlua::Value::UserData(ud) => return Some(Value::ValueType(ud.borrow::<ValueType>().ok()?.clone())),
                    // Allow passing enum values as integers
                    mlua::Value::Integer(i) => {
                        let class = type_.class();
                        if class.is_enum() {
                            if let Some(field) = class.field(c"value__") {
                                match field.type_().type_enum() {
                                    Il2CppTypeEnum_IL2CPP_TYPE_I1 => return Some(Value::I1((*i).try_into().ok()?)),
                                    Il2CppTypeEnum_IL2CPP_TYPE_U1 => return Some(Value::U1((*i).try_into().ok()?)),
                                    Il2CppTypeEnum_IL2CPP_TYPE_I2 => return Some(Value::I2((*i).try_into().ok()?)),
                                    Il2CppTypeEnum_IL2CPP_TYPE_U2 => return Some(Value::U2((*i).try_into().ok()?)),
                                    Il2CppTypeEnum_IL2CPP_TYPE_I4 => return Some(Value::I4((*i).try_into().ok()?)),
                                    Il2CppTypeEnum_IL2CPP_TYPE_U4 => return Some(Value::U4((*i).try_into().ok()?)),
                                    Il2CppTypeEnum_IL2CPP_TYPE_I8 => return Some(Value::I8(*i)),
                                    Il2CppTypeEnum_IL2CPP_TYPE_U8 => return Some(Value::U8(*i as u64)),
                                    Il2CppTypeEnum_IL2CPP_TYPE_CHAR => return Some(Value::Char((*i).try_into().ok()?)),
                                    _ => ()
                                }
                            }
                        }
                    }
                    _ => ()
                }
            }
            Il2CppTypeEnum_IL2CPP_TYPE_CLASS => {
                if let mlua::Value::UserData(ud) = value {
                    return Some(Value::Class(*ud.borrow::<Object>().ok()?))
                }
            }
            Il2CppTypeEnum_IL2CPP_TYPE_ARRAY => {
                if let mlua::Value::UserData(ud) = value {
                    return Some(Value::Array(*ud.borrow::<Array>().ok()?))
                }
            }
            Il2CppTypeEnum_IL2CPP_TYPE_GENERICINST => {
                if let mlua::Value::UserData(ud) = value {
                    return Some(Value::GenericInstance(*ud.borrow::<Object>().ok()?))
                }
            }
            Il2CppTypeEnum_IL2CPP_TYPE_OBJECT => {
                if let mlua::Value::UserData(ud) = value {
                    return Some(Value::Object(*ud.borrow::<Object>().ok()?))
                }
            }
            Il2CppTypeEnum_IL2CPP_TYPE_SZARRAY => {
                if let mlua::Value::UserData(ud) = value {
                    return Some(Value::SzArray(*ud.borrow::<Array>().ok()?))
                }
            }
            _ => ()
        }

        None
    }

    pub unsafe fn from_ffi_value(ptr: *const c_void, type_: Type) -> Result<Self, Error> {
        if ptr.is_null() {
            return Ok(Value::Void);
        }

        #[allow(non_upper_case_globals)]
        Ok(match type_.type_enum() {
            Il2CppTypeEnum_IL2CPP_TYPE_BOOLEAN => Value::Boolean(*(ptr as *const bool)),
            Il2CppTypeEnum_IL2CPP_TYPE_CHAR => Value::Char(*(ptr as *const u16)),
            Il2CppTypeEnum_IL2CPP_TYPE_I1 => Value::I1(*(ptr as *const i8)),
            Il2CppTypeEnum_IL2CPP_TYPE_U1 => Value::U1(*(ptr as *const u8)),
            Il2CppTypeEnum_IL2CPP_TYPE_I2 => Value::I2(*(ptr as *const i16)),
            Il2CppTypeEnum_IL2CPP_TYPE_U2 => Value::U2(*(ptr as *const u16)),
            Il2CppTypeEnum_IL2CPP_TYPE_I4 => Value::I4(*(ptr as *const i32)),
            Il2CppTypeEnum_IL2CPP_TYPE_U4 => Value::U4(*(ptr as *const u32)),
            Il2CppTypeEnum_IL2CPP_TYPE_I8 => Value::I8(*(ptr as *const i64)),
            Il2CppTypeEnum_IL2CPP_TYPE_U8 => Value::U8(*(ptr as *const u64)),
            Il2CppTypeEnum_IL2CPP_TYPE_R4 => Value::R4(*(ptr as *const f32)),
            Il2CppTypeEnum_IL2CPP_TYPE_R8 => Value::R8(*(ptr as *const f64)),
            Il2CppTypeEnum_IL2CPP_TYPE_I => Value::I(*(ptr as *const isize)),
            Il2CppTypeEnum_IL2CPP_TYPE_U => Value::U(*(ptr as *const usize)),
            Il2CppTypeEnum_IL2CPP_TYPE_VALUETYPE => {
                let class = type_.class();
                if class.is_enum() {
                    Self::from_ffi_value(ptr, class.field(c"value__").unwrap().type_())?
                } else {
                    Value::ValueType(ValueType::new_unchecked(ptr as *mut c_void, type_))
                }
            }
            Il2CppTypeEnum_IL2CPP_TYPE_STRING => super::String::new(*(ptr as *const _))
                .map(|v| Value::String(v))
                .unwrap_or(Value::NULL),
            Il2CppTypeEnum_IL2CPP_TYPE_PTR => Value::Pointer(Pointer::new(*(ptr as *const _), type_)),
            Il2CppTypeEnum_IL2CPP_TYPE_FNPTR => Value::Pointer(Pointer::new(*(ptr as *const _), type_)),
            Il2CppTypeEnum_IL2CPP_TYPE_BYREF => Reference::new(*(ptr as *const _), type_)
                .map(|v| Value::Reference(v))
                .unwrap_or(Value::NULL),
            Il2CppTypeEnum_IL2CPP_TYPE_ARRAY => Array::new(*(ptr as *const _))
                .map(|v| Value::Array(v))
                .unwrap_or(Value::NULL),
            Il2CppTypeEnum_IL2CPP_TYPE_SZARRAY => Array::new(*(ptr as *const _))
                .map(|v| Value::SzArray(v))
                .unwrap_or(Value::NULL),
            Il2CppTypeEnum_IL2CPP_TYPE_OBJECT => Object::new(*(ptr as *const _))
                .map(|v| Value::Object(v))
                .unwrap_or(Value::NULL),
            Il2CppTypeEnum_IL2CPP_TYPE_CLASS => Object::new(*(ptr as *const _))
                .map(|v| Value::Class(v))
                .unwrap_or(Value::NULL),
            Il2CppTypeEnum_IL2CPP_TYPE_GENERICINST => Object::new(*(ptr as *const _))
                .map(|v| Value::GenericInstance(v))
                .unwrap_or(Value::NULL),
            t => Err(Error::UnknownType(t))?,
        })
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
            Value::U8(u) => mlua::Value::Number(u as _),
            Value::R4(f) => mlua::Value::Number(f as _),
            Value::R8(f) => mlua::Value::Number(f),
            Value::I(i) => mlua::Value::Integer(i as _),
            Value::U(u) => mlua::Value::Integer(u as _),
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

#[derive(Debug)]
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


#[derive(Debug)]
#[repr(transparent)]
pub struct FfiArg<'a> {
    ptr: *const c_void,
    _lifetime: PhantomData<&'a Value>
}

impl<'a> FfiArg<'a> {
    fn new(ptr: *const c_void, _value: &'a Value) -> FfiArg {
        FfiArg {
            ptr,
            _lifetime: PhantomData
        }
    }

    pub fn get(&self) -> *const c_void {
        self.ptr
    }
}