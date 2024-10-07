use crate::{core::Hachimi, il2cpp::{symbols::get_method_addr, types::*}};

use super::{ApplicationSettingSaveLoader, SaveDataManager};

type GetRaceDynamicCameraSettingDataFn = extern "C" fn(boot_mode: *mut Il2CppObject) -> bool;
extern "C" fn GetRaceDynamicCameraSettingData(boot_mode: *mut Il2CppObject) -> bool {
    if Hachimi::instance().config.load().force_allow_dynamic_camera {
        let save_data_manager = SaveDataManager::instance();
        if save_data_manager.is_null() { return false; }

        let save_loader = SaveDataManager::get_SaveLoader(save_data_manager);
        if save_loader.is_null() { return false; }

        ApplicationSettingSaveLoader::get_IsTryRaceDynamicCamera(save_loader)
    }
    else {
        get_orig_fn!(GetRaceDynamicCameraSettingData, GetRaceDynamicCameraSettingDataFn)(boot_mode)
    }
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, RaceUtil);

    let GetRaceDynamicCameraSettingData_addr = get_method_addr(RaceUtil, c"GetRaceDynamicCameraSettingData", 1);

    new_hook!(GetRaceDynamicCameraSettingData_addr, GetRaceDynamicCameraSettingData);
}