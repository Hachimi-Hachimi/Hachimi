use std::collections::{hash_map::Entry, BTreeMap};

use fnv::FnvHashMap;
use once_cell::unsync::Lazy;

use crate::{
    core::{ext::StringExt, utils, Hachimi},
    il2cpp::{symbols::{get_method_overload_addr, unbox}, types::*}
};

use super::TextId;

// SAFETY: Localize::Get is only called from the Unity main thread.
static mut TEXTID_NAME_CACHE: Lazy<FnvHashMap<i32, String>> = Lazy::new(|| FnvHashMap::default());

/**
 * Gallop::Localize::Get
 * Used by the game to get localized strings for builtin text (mostly UI).
 * 
 * id is a value of the TextId enum
 * cy devs likes to insert stuff at arbitrary locations within the enum, changing their values
 * so we'll just map them to their actual name instead
 */
type GetFn = extern "C" fn(id: i32) -> *mut Il2CppString;
pub extern "C" fn Get(id: i32) -> *mut Il2CppString {
    let localized_data = Hachimi::instance().localized_data.load();
    if localized_data.localize_dict.is_empty() {
        return get_orig_fn!(Get, GetFn)(id);
    }

    let name = match unsafe { TEXTID_NAME_CACHE.entry(id) } {
        Entry::Occupied(e) => &*e.into_mut(),
        Entry::Vacant(e) => {
            let name = TextId::get_name(id);
            let name_str = unsafe { (*name).as_utf16str().to_string() };
            e.insert(name_str)
        },
    };

    if let Some(text) = localized_data.localize_dict.get(name) {
        text.to_il2cpp_string()
    }
    else {
        let str = get_orig_fn!(Get, GetFn)(id);
        if Hachimi::instance().config.load().translator_mode && id != 1109 && id != 1032 {
            // 1109 and 1032 seems to be debugging strings (they're annoying)
            utils::print_json_entry(name, unsafe { &(*str).as_utf16str().to_string() });
        }
        str
    }
}

pub fn dump_strings() -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();

    for obj in TextId::get_values().enumerator().map(|e| e.iter()).unwrap_or_default().expect("enum values enumerator") {
        let value: i32 = unsafe { unbox(obj) };
        let name = TextId::get_name(value);
        let name_str = unsafe { (*name).as_utf16str() };

        let res = get_orig_fn!(Get, GetFn)(value);
        if !res.is_null() {
            let res_str = unsafe { (*res).as_utf16str() };
            map.insert(name_str.to_string(), res_str.to_string());
        }
    }

    map
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, Localize);

    // Get(TextId id)
    let Get_addr = get_method_overload_addr(Localize, "Get", &[Il2CppTypeEnum_IL2CPP_TYPE_VALUETYPE]);

    new_hook!(Get_addr, Get);
}