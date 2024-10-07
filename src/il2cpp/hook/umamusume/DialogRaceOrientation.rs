use crate::{core::Hachimi, il2cpp::{symbols::get_method_addr, types::*}};

use super::RaceInfo;

type SetupAndOpenFn = extern "C" fn(
    this: *mut Il2CppObject, dialog_data: *mut Il2CppObject, on_selected: *mut Il2CppObject,
    on_cancel: *mut Il2CppObject, is_special_unlock_race: bool, race_info: *mut Il2CppObject
);
extern "C" fn SetupAndOpen(
    this: *mut Il2CppObject, dialog_data: *mut Il2CppObject, on_selected: *mut Il2CppObject,
    on_cancel: *mut Il2CppObject, is_special_unlock_race: bool, race_info: *mut Il2CppObject
) {
    let force_allow_dynamic_camera = Hachimi::instance().config.load().force_allow_dynamic_camera;
    let mut orig_race_type = None;
    if force_allow_dynamic_camera {
        orig_race_type = Some(RaceInfo::get_RaceType(race_info));
        RaceInfo::set_RaceType(race_info, 16); // spoof LoH race
    }

    get_orig_fn!(SetupAndOpen, SetupAndOpenFn)(
        this, dialog_data, on_selected, on_cancel, is_special_unlock_race, race_info
    );

    if let Some(race_type) = orig_race_type {
        RaceInfo::set_RaceType(race_info, race_type);
    }
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, DialogRaceOrientation);

    let SetupAndOpen_addr = get_method_addr(DialogRaceOrientation, c"SetupAndOpen", 5);

    new_hook!(SetupAndOpen_addr, SetupAndOpen);
}