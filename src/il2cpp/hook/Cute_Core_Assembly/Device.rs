use crate::il2cpp::{symbols::get_method_addr, types::*};

extern "C" fn IsIllegalUser() -> bool { false }

pub fn init(Cute_Core_Assembly: *const Il2CppImage) {
    get_class_or_return!(Cute_Core_Assembly, "Cute.Core", Device);

    let IsIllegalUser_addr = get_method_addr(Device, c"IsIllegalUser", 0);

    new_hook!(IsIllegalUser_addr, IsIllegalUser);
}