use std::path::Path;

use fnv::FnvHashMap;

use crate::{
    core::{ext::{StringExt, Utf16StringExt}, Hachimi},
    il2cpp::{symbols::{get_method_addr, IList}, types::*}
};

type LoadCSVFn = extern "C" fn(path: *mut Il2CppString) -> *mut Il2CppObject;
extern "C" fn LoadCSV(path: *mut Il2CppString) -> *mut Il2CppObject {
    // Live/MusicScores/mXXXX/mXXXX_lyrics
    let path_str = unsafe { (*path).to_utf16str() };

    // ArrayList<ArrayList<String>>
    let array_list_obj = get_orig_fn!(LoadCSV, LoadCSVFn)(path);

    let mut dict_path = Path::new("lyrics").join(path_str.path_filename().to_string());
    dict_path.set_extension("json");
    let localized_data = Hachimi::instance().localized_data.load();
    let Some(dict): Option<FnvHashMap<String, String>> = localized_data.load_assets_dict(Some(&dict_path)) else {
        return array_list_obj;
    };

    let Some(array_list) = IList::new(array_list_obj) else {
        return array_list_obj;
    };

    for row_obj in array_list.iter() {
        let Some(row) = IList::<*mut Il2CppString>::new(row_obj) else {
            return array_list_obj;
        };
        let Some(time) = row.get(0) else { continue };
        //let Some(lyrics) = row.get(1) else { continue };
        
        let time_str = unsafe { (*time).to_utf16str().to_string() };
        if let Some(new_lyrics) = dict.get(&time_str) {
            row.set(1, new_lyrics.to_il2cpp_string());
        }
    }

    array_list_obj
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, "Gallop.Live", LyricsController);

    let LoadCSV_addr = get_method_addr(LyricsController, cstr!("LoadCSV"), 1);

    new_hook!(LoadCSV_addr, LoadCSV);
}