use crate::il2cpp::{symbols::get_method_addr, types::*};

type GetSafetyNetStatusFn = extern "C" fn(
    api_key: *mut Il2CppString, nonce: *mut Il2CppString,
    on_success: *mut Il2CppDelegate, on_error: *mut Il2CppDelegate
);
pub extern "C" fn GetSafetyNetStatus(
    api_key: *mut Il2CppString, nonce: *mut Il2CppString,
    on_success: *mut Il2CppDelegate, _on_error: *mut Il2CppDelegate
) {
    get_orig_fn!(GetSafetyNetStatus, GetSafetyNetStatusFn)(api_key, nonce, on_success, on_success);
}

pub fn init(Cute_Core_Assembly: *const Il2CppImage) {
    get_class_or_return!(Cute_Core_Assembly, "Cute.Core", SafetyNet);

    let GetSafetyNetStatus_addr = get_method_addr(SafetyNet, c"GetSafetyNetStatus", 4);

    new_hook!(GetSafetyNetStatus_addr, GetSafetyNetStatus);
}