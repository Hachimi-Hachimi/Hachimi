use mlua::{FromLuaMulti, Lua, Result, UserData, UserDataFields, UserDataMethods};

use super::{types::*, wrapper::{Domain, NativePointer, Type, ValueType}};

struct Il2Cpp;

impl UserData for Il2Cpp {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("alloc", |_, _, size: usize|
            Ok(NativePointer::alloc(size))
        );

        methods.add_method("new", |lua, _, (type_name, args): (String, mlua::MultiValue)| {
            match type_name.as_str() {
                "ValueType" => {
                    let (ptr, type_): (NativePointer, Type) = FromLuaMulti::from_lua_multi(args, lua)?;
                    Ok(ValueType::new(ptr, type_))
                },
                _ => Err(mlua::Error::external(format!("Unknown type name: {}", type_name)))
            }
        });
    }

    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field("domain", Domain::get());

        fields.add_field("TypeEnum", TypeEnum);
    }
}

struct TypeEnum;

impl UserData for TypeEnum {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field("End", Il2CppTypeEnum_IL2CPP_TYPE_END);
        fields.add_field("Void", Il2CppTypeEnum_IL2CPP_TYPE_VOID);
        fields.add_field("Boolean", Il2CppTypeEnum_IL2CPP_TYPE_BOOLEAN);
        fields.add_field("Char", Il2CppTypeEnum_IL2CPP_TYPE_CHAR);
        fields.add_field("I1", Il2CppTypeEnum_IL2CPP_TYPE_I1);
        fields.add_field("U1", Il2CppTypeEnum_IL2CPP_TYPE_U1);
        fields.add_field("I2", Il2CppTypeEnum_IL2CPP_TYPE_I2);
        fields.add_field("U2", Il2CppTypeEnum_IL2CPP_TYPE_U2);
        fields.add_field("I4", Il2CppTypeEnum_IL2CPP_TYPE_I4);
        fields.add_field("U4", Il2CppTypeEnum_IL2CPP_TYPE_U4);
        fields.add_field("I8", Il2CppTypeEnum_IL2CPP_TYPE_I8);
        fields.add_field("U8", Il2CppTypeEnum_IL2CPP_TYPE_U8);
        fields.add_field("R4", Il2CppTypeEnum_IL2CPP_TYPE_R4);
        fields.add_field("R8", Il2CppTypeEnum_IL2CPP_TYPE_R8);
        fields.add_field("String", Il2CppTypeEnum_IL2CPP_TYPE_STRING);
        fields.add_field("Ptr", Il2CppTypeEnum_IL2CPP_TYPE_PTR);
        fields.add_field("Byref", Il2CppTypeEnum_IL2CPP_TYPE_BYREF);
        fields.add_field("ValueType", Il2CppTypeEnum_IL2CPP_TYPE_VALUETYPE);
        fields.add_field("Class", Il2CppTypeEnum_IL2CPP_TYPE_CLASS);
        fields.add_field("Var", Il2CppTypeEnum_IL2CPP_TYPE_VAR);
        fields.add_field("Array", Il2CppTypeEnum_IL2CPP_TYPE_ARRAY);
        fields.add_field("GenericInst", Il2CppTypeEnum_IL2CPP_TYPE_GENERICINST);
        fields.add_field("TypedByRef", Il2CppTypeEnum_IL2CPP_TYPE_TYPEDBYREF);
        fields.add_field("I", Il2CppTypeEnum_IL2CPP_TYPE_I);
        fields.add_field("U", Il2CppTypeEnum_IL2CPP_TYPE_U);
        fields.add_field("FnPtr", Il2CppTypeEnum_IL2CPP_TYPE_FNPTR);
        fields.add_field("Object", Il2CppTypeEnum_IL2CPP_TYPE_OBJECT);
        fields.add_field("SzArray", Il2CppTypeEnum_IL2CPP_TYPE_SZARRAY);
        fields.add_field("MVar", Il2CppTypeEnum_IL2CPP_TYPE_MVAR);
        fields.add_field("CmodReqd", Il2CppTypeEnum_IL2CPP_TYPE_CMOD_REQD);
        fields.add_field("CmodOpt", Il2CppTypeEnum_IL2CPP_TYPE_CMOD_OPT);
        fields.add_field("Internal", Il2CppTypeEnum_IL2CPP_TYPE_INTERNAL);
        fields.add_field("Modifier", Il2CppTypeEnum_IL2CPP_TYPE_MODIFIER);
        fields.add_field("Sentinel", Il2CppTypeEnum_IL2CPP_TYPE_SENTINEL);
        fields.add_field("Pinned", Il2CppTypeEnum_IL2CPP_TYPE_PINNED);
        fields.add_field("Enum", Il2CppTypeEnum_IL2CPP_TYPE_ENUM);
        fields.add_field("Il2CppTypeIndex", Il2CppTypeEnum_IL2CPP_TYPE_IL2CPP_TYPE_INDEX);
    }
}

pub fn init(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set("Il2Cpp", Il2Cpp)?;

    Ok(())
}