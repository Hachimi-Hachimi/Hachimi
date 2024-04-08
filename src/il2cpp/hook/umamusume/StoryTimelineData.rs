use std::ptr::null_mut;

use serde::Deserialize;
use widestring::Utf16Str;

use crate::{
    core::{ext::{StringExt, Utf16StringExt}, utils, Hachimi}, 
    il2cpp::{
        hook::UnityEngine_AssetBundleModule::AssetBundle::ASSET_PATH_PREFIX,
        symbols::{get_field_from_name, get_field_object_value, set_field_object_value, IList},
        types::*
    }
};

use super::{StoryTimelineBlockData, StoryTimelineTextClipData, StoryTimelineTrackData};

const CLIP_TEXT_LINE_WIDTH: i32 = 21;
const CLIP_TEXT_LINE_COUNT: i32 = 3;
const CLIP_TEXT_FONT_SIZE: i32 = 42;

static mut CLASS: *mut Il2CppClass = null_mut();
pub fn class() -> *mut Il2CppClass {
    unsafe { CLASS }
}

static mut TITLE_FIELD: *mut FieldInfo = null_mut();
fn set_Title(this: *mut Il2CppObject, value: *mut Il2CppString) {
    set_field_object_value(this, unsafe { TITLE_FIELD }, value);
}

// List<StoryTimelineBlockData>
static mut BLOCKLIST_FIELD: *mut FieldInfo = null_mut();
fn get_BlockList(this: *mut Il2CppObject) -> *mut Il2CppObject {
    get_field_object_value(this, unsafe { BLOCKLIST_FIELD })
}

// (Aliases are there for tlg compatibility)
#[derive(Deserialize)]
struct StoryTimelineDataDict {
    #[serde(alias = "Title")]
    title: Option<String>,
    
    #[serde(alias = "TextBlockList")]
    #[serde(default)]
    text_block_list: Vec<TextBlockDict>
}

#[derive(Deserialize)]
struct TextBlockDict {
    #[serde(alias = "Name")]
    name: Option<String>,

    #[serde(alias = "Text")]
    text: Option<String>,

    #[serde(alias = "ChoiceDataList")]
    #[serde(default)]
    choice_data_list: Vec<String>,

    #[serde(alias = "ColorTextInfoList")]
    #[serde(default)]
    color_text_info_list: Vec<String>,

    //new_clip_length: Option<i32>
}

// hook::UnityEngine_AssetBundleModule::AssetBundle
// name:
// - assets/_gallopresources/bundle/resources/home/data/xxxxx/yy/hometimeline_xxxxx_yy_zzzzzzz.asset
// - assets/_gallopresources/bundle/resources/story/data/xx/yyyy/storytimeline_xxyyyyzzz.asset
// TODO: Implement clip length adjustment
pub fn on_LoadAsset(asset: &mut *mut Il2CppObject, name: &Utf16Str) {
    if !name.starts_with(ASSET_PATH_PREFIX) {
        // ???
        return;
    }

    let base_path = name[ASSET_PATH_PREFIX.len()..].path_basename();
    let dict_path = base_path.to_string() + ".json";

    let localized_data = Hachimi::instance().localized_data.load();
    let Some(dict): Option<StoryTimelineDataDict> = localized_data.load_assets_dict(Some(&dict_path)) else {
        return;
    };
    debug!("{}", dict_path);
    /*
    let fps = if hachimi.target_fps != -1 {
        hachimi.target_fps
    }
    else {
        30
    };
    */

    let this = *asset;
    if let Some(title) = &dict.title {
        set_Title(this, title.to_il2cpp_string());
    }

    let Some(block_list) = IList::<*mut Il2CppObject>::new(get_BlockList(this)) else {
        return;
    };

    // enumerenumerenumer
    for (ii, block_data) in block_list.iter().enumerate() {
        // cy leaves a single empty text block at the start of every story for some reason
        if ii == 0 { continue; }
        let i = ii - 1;

        let Some(text_block_dict) = dict.text_block_list.get(i) else {
            warn!("text block {} not found in dict: {}", i, dict_path);
            break;
        };

        let text_track = StoryTimelineBlockData::get_TextTrack(block_data);
        if text_track.is_null() {
            continue;
        }

        let clip_list_obj = StoryTimelineTrackData::get_ClipList(text_track);
        let Some(clip_list) = IList::<*mut Il2CppObject>::new(clip_list_obj) else {
            continue;
        };
        for clip_data in clip_list.iter() {
            let class = unsafe { (*clip_data).klass() };
            if class != StoryTimelineTextClipData::class() {
                continue;
            }

            if let Some(name) = &text_block_dict.name {
                StoryTimelineTextClipData::set_Name(clip_data, name.to_il2cpp_string());
            }
            if let Some(text) = &text_block_dict.text {
                let size = StoryTimelineTextClipData::get_Size(this);
                let text_str = if size == StoryTimelineTextClipData::FontSize_Default {
                    if let Some(fitted) = utils::wrap_fit_text(text,
                        CLIP_TEXT_LINE_WIDTH, CLIP_TEXT_LINE_COUNT, CLIP_TEXT_FONT_SIZE
                    ) {
                        fitted.to_il2cpp_string()
                    }
                    else {
                        text.to_il2cpp_string()
                    }
                }
                else {
                    // dont wanna mess other sizes for now, but at least wrap the text if enabled
                    // TODO: wrap fit text with line width relative to font size value
                    // (which stories is it even used in...?)
                    if let Some(wrapped) = utils::wrap_text(text, CLIP_TEXT_LINE_WIDTH) {
                        wrapped.to_il2cpp_string()
                    }
                    else {
                        text.to_il2cpp_string()
                    }
                };
                StoryTimelineTextClipData::set_Text(clip_data, text_str);
            }

            // IList::new checks for null, no need to do so explicitly
            let choice_data_list_obj = StoryTimelineTextClipData::get_ChoiceDataList(clip_data);
            if let Some(choice_data_list) = IList::new(choice_data_list_obj) {
                for (j, choice_data) in choice_data_list.iter().enumerate() {
                    if let Some(text) = text_block_dict.choice_data_list.get(j) {
                        if !text.is_empty() {
                            StoryTimelineTextClipData::ChoiceData::set_Text(choice_data, text.to_il2cpp_string())
                        }
                    }
                    else {
                        warn!("choice data {} of block {} not found in dict: {}", j, i, dict_path);
                    }
                }
            }

            let color_text_info_list_obj = StoryTimelineTextClipData::get_ColorTextInfoList(clip_data);
            if let Some(color_text_info_list) = IList::new(color_text_info_list_obj) {
                for (j, color_text_info) in color_text_info_list.iter().enumerate() {
                    if let Some(text) = text_block_dict.color_text_info_list.get(j) {
                        if !text.is_empty() {
                            StoryTimelineTextClipData::ColorTextInfo::set_Text(color_text_info, text.to_il2cpp_string())
                        }
                    }
                    else {
                        warn!("color text info {} of block {} not found in dict: {}", j, i, dict_path);
                    }
                }
            }
        }
    }
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, StoryTimelineData);

    unsafe {
        CLASS = StoryTimelineData;
        TITLE_FIELD = get_field_from_name(StoryTimelineData, cstr!("Title"));
        BLOCKLIST_FIELD = get_field_from_name(StoryTimelineData, cstr!("BlockList"));
    }
}