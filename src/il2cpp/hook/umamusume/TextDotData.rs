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

static mut DOTBLOCKDATA_CLASS: *mut Il2CppClass = 0 as _;
fn DotBlockData_class() -> *mut Il2CppClass {
    unsafe { DOTBLOCKDATA_CLASS }
}

pub fn on_LoadAsset(_bundle: *mut Il2CppObject, this: *mut Il2CppObject, _name: &Utf16Str) {
    if Hachimi::instance().localized_data.load().config.remove_ruby {
        let empty_array = il2cpp_array_new(DotBlockData_class(), 0);
        set_DataArray(this, empty_array);
    }
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, "", TextDotData);

    unsafe {
        CLASS = TextDotData;
        DATAARRAY_FIELD = get_field_from_name(TextDotData, c"DataArray");
    }

    // Putting nested class inside parent module due to lack of usage
    find_nested_class_or_return!(TextDotData, DotBlockData);

    unsafe {
        DOTBLOCKDATA_CLASS = DotBlockData;
    }
}