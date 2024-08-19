use widestring::Utf16Str;

use crate::{
    core::Hachimi,
    il2cpp::{
        api::il2cpp_array_new,
        symbols::{get_field_from_name, set_field_object_value},
        types::*
    }
};

static mut CLASS: *mut Il2CppClass = 0 as _;
pub fn class() -> *mut Il2CppClass {
    unsafe { CLASS }
}

static mut DATAARRAY_FIELD: *mut FieldInfo = 0 as _;
fn set_DataArray(this: *mut Il2CppObject, value: *mut Il2CppArray) {
    set_field_object_value(this, unsafe { DATAARRAY_FIELD }, value);
}

static mut RUBYBLOCKDATA_CLASS: *mut Il2CppClass = 0 as _;
fn RubyBlockData_class() -> *mut Il2CppClass {
    unsafe { RUBYBLOCKDATA_CLASS }
}

pub fn on_LoadAsset(_bundle: *mut Il2CppObject, this: *mut Il2CppObject, _name: &Utf16Str) {
    if Hachimi::instance().localized_data.load().config.remove_ruby {
        let empty_array = il2cpp_array_new(RubyBlockData_class(), 0);
        set_DataArray(this, empty_array);
    }
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, "", TextRubyData);

    unsafe {
        CLASS = TextRubyData;
        DATAARRAY_FIELD = get_field_from_name(TextRubyData, c"DataArray");
    }

    // Putting nested class inside parent module due to lack of usage
    find_nested_class_or_return!(TextRubyData, RubyBlockData);

    unsafe {
        RUBYBLOCKDATA_CLASS = RubyBlockData;
    }
}