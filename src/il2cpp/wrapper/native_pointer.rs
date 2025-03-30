use std::{ffi::CStr, os::raw::c_void};

use mlua::{FromLua, Lua, MetaMethod, UserData, UserDataMethods};

#[derive(Debug, Clone)]
pub enum NativePointer {
    Raw(*mut c_void),
    Owned(Vec<u8>)
}

impl NativePointer {
    pub fn alloc(size: usize) -> Self {
        Self::Owned(vec![0; size])
    }

    pub fn get(&self) -> *mut c_void {
        match self {
            NativePointer::Raw(p) => *p,
            NativePointer::Owned(v) => v.as_ptr() as _,
        }
    }
}

impl From<*mut c_void> for NativePointer {
    fn from(value: *mut c_void) -> Self {
        Self::Raw(value)
    }
}

impl UserData for NativePointer {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(MetaMethod::ToString, |lua, this, ()| {
            lua.create_string(format!("pointer: {:#x}", this.get() as usize))
        });

        methods.add_method("address", |_, this, ()| Ok(this.get() as usize));

        methods.add_method("read_i8", |_, this, ()| unsafe { Ok(*(this.get() as *const i8)) });
        methods.add_method("read_u8", |_, this, ()| unsafe { Ok(*(this.get() as *const u8)) });
        methods.add_method("read_i16", |_, this, ()| unsafe { Ok(*(this.get() as *const i16)) });
        methods.add_method("read_u16", |_, this, ()| unsafe { Ok(*(this.get() as *const u16)) });
        methods.add_method("read_i32", |_, this, ()| unsafe { Ok(*(this.get() as *const i32)) });
        methods.add_method("read_u32", |_, this, ()| unsafe { Ok(*(this.get() as *const u32)) });
        methods.add_method("read_i64", |_, this, ()| unsafe { Ok(*(this.get() as *const i64)) });
        methods.add_method("read_u64", |_, this, ()| unsafe { Ok(*(this.get() as *const u64)) });
        methods.add_method("read_f32", |_, this, ()| unsafe { Ok(*(this.get() as *const f32)) });
        methods.add_method("read_f64", |_, this, ()| unsafe { Ok(*(this.get() as *const f64)) });

        methods.add_method("write_i8", |_, this, value: i8| {
            unsafe {
                *(this.get() as *mut i8) = value;
                Ok(())
            }
        });

        methods.add_method("write_u8", |_, this, value: u8| {
            unsafe {
                *(this.get() as *mut u8) = value;
                Ok(())
            }
        });

        methods.add_method("write_i16", |_, this, value: i16| {
            unsafe {
                *(this.get() as *mut i16) = value;
                Ok(())
            }
        });

        methods.add_method("write_u16", |_, this, value: u16| {
            unsafe {
                *(this.get() as *mut u16) = value;
                Ok(())
            }
        });

        methods.add_method("write_i32", |_, this, value: i32| {
            unsafe {
                *(this.get() as *mut i32) = value;
                Ok(())
            }
        });

        methods.add_method("write_u32", |_, this, value: u32| {
            unsafe {
                *(this.get() as *mut u32) = value;
                Ok(())
            }
        });

        methods.add_method("write_i64", |_, this, value: i64| {
            unsafe {
                *(this.get() as *mut i64) = value;
                Ok(())
            }
        });

        methods.add_method("write_u64", |_, this, value: u64| {
            unsafe {
                *(this.get() as *mut u64) = value;
                Ok(())
            }
        });

        methods.add_method("write_f32", |_, this, value: f32| {
            unsafe {
                *(this.get() as *mut f32) = value;
                Ok(())
            }
        });

        methods.add_method("write_f64", |_, this, value: f64| {
            unsafe {
                *(this.get() as *mut f64) = value;
                Ok(())
            }
        });

        methods.add_method("read_bytes", |_, this, len: usize| {
            unsafe {
                let ptr = this.get() as *const u8;
                let bytes = std::slice::from_raw_parts(ptr, len);
                Ok(bytes.to_vec())
            }
        });

        methods.add_method("write_bytes", |_, this, bytes: Vec<u8>| {
            unsafe {
                let ptr = this.get() as *mut u8;
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
                Ok(())
            }
        });

        methods.add_method("read_string", |lua, this, ()| {
            unsafe {
                let c_str = CStr::from_ptr(this.get() as _);
                let bytes = c_str.to_bytes();
                lua.create_string(bytes)
            }
        });

        methods.add_method("write_string", |_, this, string: mlua::String| {
            unsafe {
                let bytes = string.as_bytes_with_nul();
                let ptr = this.get() as *mut u8;
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
                Ok(())
            }
        });
    }
}

impl FromLua for NativePointer {
    fn from_lua(value: mlua::Value, _: &Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "NativePointer".to_owned(),
                message: None
            })
        }
    }
}