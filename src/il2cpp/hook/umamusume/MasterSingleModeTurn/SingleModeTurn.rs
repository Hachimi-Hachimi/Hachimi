use crate::il2cpp::{symbols::{get_field_from_name, get_field_value}, types::*};

static mut MONTH_FIELD: *mut FieldInfo = 0 as _;
pub fn get_Month(this: *mut Il2CppObject) -> i32 {
    get_field_value(this, unsafe { MONTH_FIELD })
}

static mut HALF_FIELD: *mut FieldInfo = 0 as _;
pub fn get_Half(this: *mut Il2CppObject) -> i32 {
    get_field_value(this, unsafe { HALF_FIELD })
}

pub fn init(MasterSingleModeTurn: *mut Il2CppClass) {
    find_nested_class_or_return!(MasterSingleModeTurn, SingleModeTurn);

    unsafe {
        MONTH_FIELD = get_field_from_name(SingleModeTurn, c"Month");
        HALF_FIELD = get_field_from_name(SingleModeTurn, c"Half");
    }
}