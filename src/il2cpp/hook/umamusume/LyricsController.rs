use std::path::Path;

use fnv::FnvHashMap;

use crate::{
    core::{ext::Utf16StringExt, Hachimi},
    il2cpp::{
        ext::{Il2CppStringExt, StringExt},
        symbols::{get_field_from_name, get_field_object_value, get_method_addr, Array, Dictionary},
        types::*
    }
};

static mut LYRICSDATADIC_FIELD: *mut FieldInfo = 0 as _;
fn get__lyricsDataDic(this: *mut Il2CppObject) -> Dictionary<i32, Array<LyricsData>> {
    Dictionary::from(get_field_object_value(this, unsafe { LYRICSDATADIC_FIELD }))
}

// dead code needed here, currently unused in-game
#[repr(i32)]
#[allow(dead_code)]
enum AdditionalSetting {
    None = 0,
    SheetVariationId = 1
}

#[repr(C)]
struct LyricsData {
    time: f32,
    lyrics: *mut Il2CppString,
    additionalsetting_type: AdditionalSetting,
    additionalsetting_value: i32
}

type LoadLyricsFn = extern "C" fn(this: *mut Il2CppObject, id: i32, path: *mut Il2CppString) -> bool;
extern "C" fn LoadLyrics(this: *mut Il2CppObject, id: i32, path: *mut Il2CppString) -> bool {
    if !get_orig_fn!(LoadLyrics, LoadLyricsFn)(this, id, path) {
        return false;
    }

    // Live/MusicScores/mXXXX/mXXXX_lyrics
    let path_str = unsafe { (*path).as_utf16str() };

    let mut dict_path = Path::new("lyrics").join(path_str.path_filename().to_string());
    dict_path.set_extension("json");
    let localized_data = Hachimi::instance().localized_data.load();
    let Some(dict): Option<FnvHashMap<i32, String>> = localized_data.load_assets_dict(Some(&dict_path)) else {
        return true;
    };
    // dont let pbork interactive know about this
    let secs_dict: FnvHashMap<i32, String> = dict.into_iter()
        .map(|(time, lyrics)| unsafe { (std::mem::transmute(time as f32 / 1000.0), lyrics) })
        .collect();

    let lyrics_data_dict = get__lyricsDataDic(this);
    let Some(lyrics_data_array) = lyrics_data_dict.get(&id) else {
        return true;
    };
    for lyrics_data in unsafe { lyrics_data_array.as_slice().iter_mut() } {
        // transmute to an i32 so we can do an exact match lookup in the map
        let time: i32 = unsafe { std::mem::transmute(lyrics_data.time) };
        if let Some(text) = secs_dict.get(&time) {
            lyrics_data.lyrics = text.to_il2cpp_string();
        }
    }

    true
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, "Gallop.Live", LyricsController);

    let LoadLyrics_addr = get_method_addr(LyricsController, c"LoadLyrics", 2);

    new_hook!(LoadLyrics_addr, LoadLyrics);

    unsafe {
        LYRICSDATADIC_FIELD = get_field_from_name(LyricsController, c"_lyricsDataDic");
    }
}