use serde::{Deserialize, Serialize};

use crate::{core::Hachimi, il2cpp::{symbols::{get_field_from_name, get_method_addr, set_field_value}, types::*}};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[repr(i32)]
pub enum SpringUpdateMode {
    ModeNormal,
    Mode60FPS,
    SkipFrame,
    SkipFramePostAlways
}

static mut UPDATEMODE_FIELD: *mut FieldInfo = 0 as _;
fn set_UpdateMode(this: *mut Il2CppObject, value: &SpringUpdateMode) {
    set_field_value(this, unsafe { UPDATEMODE_FIELD }, value);
}

type InitFn = extern "C" fn(this: *mut Il2CppObject);
extern "C" fn Init(this: *mut Il2CppObject) {
    get_orig_fn!(Init, InitFn)(this);

    if let Some(mode) = Hachimi::instance().config.load().physics_update_mode.as_ref() {
        set_UpdateMode(this, mode);
    }
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, CySpringController);

    let Init_addr = get_method_addr(CySpringController, c"Init", 0);

    new_hook!(Init_addr, Init);

    unsafe {
        UPDATEMODE_FIELD = get_field_from_name(CySpringController, c"<UpdateMode>k__BackingField");
    }
}