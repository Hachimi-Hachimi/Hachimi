use std::{ffi::{CStr, CString}, sync::Arc};

use mlua::{UserDataMethods, Value as LuaValue, FromLua};

use crate::il2cpp::types::Il2CppTypeEnum;

use super::{BoundField, BoundMethod, Class, MethodFindMode, Object, Value, ValueType};

/// Used to represent a value that's bound to a field or a method
#[derive(Debug, Clone)]
pub enum BoundValue {
    Object(Object),
    ValueType(ValueType)
}

impl BoundValue {
    pub fn from_value(value: Value) -> Option<BoundValue> {
        Some(match value {
            Value::Object(o) => BoundValue::Object(o),
            Value::ValueType(v) => BoundValue::ValueType(v),
            _ => return None
        })
    }
}

impl From<Object> for BoundValue {
    fn from(value: Object) -> Self {
        Self::Object(value)
    }
}

impl From<ValueType> for BoundValue {
    fn from(value: ValueType) -> Self {
        Self::ValueType(value)
    }
}


pub trait BindableValue: Into<BoundValue> + Clone {
    fn class(&self) -> Class;

    fn field(&self, name: &CStr) -> Option<BoundField> {
        self.class().field(name).map(|f| f.bind(self.clone()))
    }

    fn method(&self, name: &str, mode: MethodFindMode) -> Option<BoundMethod> {
        self.class().method(name, mode).map(|f| f.bind(self.clone()))
    }

    fn method_by_name(&self, name: &str) -> Option<BoundMethod> {
        self.method(name, MethodFindMode::NameOnly)
    }

    fn method_with_param_count(&self, name: &str, param_count: u8) -> Option<BoundMethod> {
        self.method(name, MethodFindMode::WithParamCount(param_count))
    }

    fn method_with_param_list(&self, name: &str, param_list: &[Il2CppTypeEnum]) -> Option<BoundMethod> {
        self.method(name, MethodFindMode::WithParamList(param_list))
    }

    fn add_bindable_value_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("field", |_, this, name: CString| Ok(this.field(&name)));
        methods.add_method("method", |lua, class, (name, arg_count_or_list): (String, LuaValue)| {
            let list: Vec<Il2CppTypeEnum>;
            let mode = match arg_count_or_list {
                LuaValue::Nil => MethodFindMode::NameOnly,

                LuaValue::Integer(_) => MethodFindMode::WithParamCount(
                    u8::from_lua(arg_count_or_list, lua)?
                ),

                LuaValue::Table(_) => {
                    list = Vec::from_lua(arg_count_or_list, lua)?;
                    MethodFindMode::WithParamList(&list)
                },

                _ => return Err(mlua::Error::BadArgument {
                    to: Some("method".to_owned()),
                    pos: 3,
                    name: Some("argCountOrList".to_owned()),
                    cause: Arc::new(mlua::Error::external("Invalid type")),
                })
            };

            Ok(class.method(&name, mode))
        });
    }
}