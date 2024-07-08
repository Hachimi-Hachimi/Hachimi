use crate::{core::utils, il2cpp::{symbols::get_method_addr, types::*}};

type LineHeadWrapCommonFn = extern "C" fn(s: *mut Il2CppString, line_char_count: i32, handling_type: i32, is_match_delegate: *mut Il2CppDelegate) -> *mut Il2CppString;
extern "C" fn LineHeadWrapCommon(s: *mut Il2CppString, line_char_count: i32, handling_type: i32, is_match_delegate: *mut Il2CppDelegate) -> *mut Il2CppString {
    if utils::game_str_has_newline(s) {
        // assume prewrapped, let the game handle it
        return get_orig_fn!(LineHeadWrapCommon, LineHeadWrapCommonFn)(s, line_char_count, handling_type, is_match_delegate);
    }

    if let Some(wrapped) = utils::wrap_text_il2cpp(s, line_char_count) {
        return wrapped;
    }
    get_orig_fn!(LineHeadWrapCommon, LineHeadWrapCommonFn)(s, line_char_count, handling_type, is_match_delegate)
}

static mut GOTOTITLEONERROR_ADDR: usize = 0;
impl_addr_wrapper_fn!(GotoTitleOnError, GOTOTITLEONERROR_ADDR, (), text: *mut Il2CppString);

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, GallopUtil);

    let LineHeadWrapCommon_addr = get_method_addr(GallopUtil, c"LineHeadWrapCommon", 4);

    new_hook!(LineHeadWrapCommon_addr, LineHeadWrapCommon);

    unsafe {
        GOTOTITLEONERROR_ADDR = get_method_addr(GallopUtil, c"GotoTitleOnError", 1);
    }
}