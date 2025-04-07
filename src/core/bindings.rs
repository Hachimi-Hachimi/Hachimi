use std::sync::Arc;

use mlua::{Lua, MultiValue, Result};

use crate::il2cpp;

pub fn init(print_tag: impl Into<String>) -> Result<Arc<Lua>> {
    let print_tag = print_tag.into();
    let lua = Arc::new(Lua::new());
    lua.set_app_data(Arc::downgrade(&lua));

    fn get_print_message(values: &MultiValue) -> Result<String> {
        let mut message = String::new();
        let mut iter = values.iter();

        // Write the first arg
        if let Some(arg) = iter.next() {
            message += &arg.to_string()?;
        }

        // Write the rest
        for arg in iter {
            message += "\t";
            message += &arg.to_string()?;
        }

        Ok(message)
    }

    let globals = lua.globals();

    {
        let print_tag = print_tag.clone();
        globals.set("print", lua.create_function(move |_, values: MultiValue| {
            let message = get_print_message(&values)?;
            info!(target: "[lua]", "[{}] {}", print_tag, message);
            Ok(())
        })?)?;
    }

    globals.set("warn", lua.create_function(move |_, values: MultiValue| {
        let message = get_print_message(&values)?;
        warn!(target: "[lua]", "[{}] {}", print_tag, message);
        Ok(())
    })?)?;

    il2cpp::bindings::init(&lua)?;
    Ok(lua)
}