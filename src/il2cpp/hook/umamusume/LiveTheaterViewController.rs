use crate::{core::{utils::notify_error, Hachimi}, il2cpp::{symbols::get_method_addr, types::*}};

static mut CHANGELIVE_ONSUCCESS_ADDR: usize = 0;
impl_addr_wrapper_fn!(ChangeLive_onSuccess, CHANGELIVE_ONSUCCESS_ADDR, (), this: *mut Il2CppObject, res: *mut Il2CppObject);

type ChangeLiveFn = extern "C" fn(this: *mut Il2CppObject);
extern "C" fn ChangeLive(this: *mut Il2CppObject) {
    if Hachimi::instance().config.load().live_theater_allow_same_chara {
        if unsafe { CHANGELIVE_ONSUCCESS_ADDR } == 0 {
            return notify_error("BUG: Please turn off 'Live theater allow same chara' \
                and report this to the developer.");
        }
        // As of the current version, res is unused so we can safely pass NULL
        return ChangeLive_onSuccess(this, 0 as _);
    }

    get_orig_fn!(ChangeLive, ChangeLiveFn)(this);
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, LiveTheaterViewController);

    let ChangeLive_addr = get_method_addr(LiveTheaterViewController, c"ChangeLive", 0);

    new_hook!(ChangeLive_addr, ChangeLive);

    unsafe {
        CHANGELIVE_ONSUCCESS_ADDR = get_method_addr(LiveTheaterViewController, c"<ChangeLive>b__48_1", 1);
    }
}