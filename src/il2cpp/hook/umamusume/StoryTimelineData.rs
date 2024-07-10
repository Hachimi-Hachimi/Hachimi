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
const CLIP_TEXT_FONT_SIZE_DEFAULT: i32 = 42;
/*
const CLIP_TEXT_FONT_SIZE_LARGE: i32 = 84;
const CLIP_TEXT_FONT_SIZE_SMALL: i32 = 32;
const CLIP_TEXT_FONT_SIZE_BOLD_CAPTION: i32 = 64;
*/

// probably?
const STORY_VIEW_CLIP_TEXT_LINE_WIDTH: i32 = 32;

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
pub fn on_LoadAsset(_bundle: *mut Il2CppObject, this: *mut Il2CppObject, name: &Utf16Str) {
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
    let is_story_view = base_path.starts_with("story/data/") && (
        base_path[11..].starts_with("02/") ||
        base_path[11..].starts_with("04/") ||
        base_path[11..].starts_with("09/")
    );

    if let Some(title) = &dict.title {
        set_Title(this, title.to_il2cpp_string());
    }

    let Some(block_list) = IList::new(get_BlockList(this)) else {
        return;
    };

    // Init wrapping parameters
    let mut line_count = CLIP_TEXT_LINE_COUNT;
    if let Some(offset) = localized_data.config.story_line_count_offset {
        line_count += offset;
    }

    let mut font_size = CLIP_TEXT_FONT_SIZE_DEFAULT;
    let mut line_width = CLIP_TEXT_LINE_WIDTH;
    let mut story_view_line_width = STORY_VIEW_CLIP_TEXT_LINE_WIDTH;
    if let Some(mult) = localized_data.config.text_frame_font_size_multiplier {
        font_size = (font_size as f32 * mult).round() as i32;
        line_width = (line_width as f32 / mult).round() as i32;
        story_view_line_width = (story_view_line_width as f32 / mult).round() as i32;
    }

    for (mut i, block_data) in block_list.iter().enumerate() {
        // cy leaves a single empty text block at the start of every story for some reason
        if i == 0 { continue; }
        i -= 1;

        let Some(text_block_dict) = dict.text_block_list.get(i) else {
            warn!("text block {} not found in dict: {}", i, dict_path);
            break;
        };

        let text_track = StoryTimelineBlockData::get_TextTrack(block_data);
        if text_track.is_null() {
            continue;
        }

        let clip_list_obj = StoryTimelineTrackData::get_ClipList(text_track);
        let Some(clip_list) = <IList>::new(clip_list_obj) else {
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
                let text_str: *mut Il2CppString;
                if is_story_view {
                    // Sizing tags are not used at all in main stories, simply wrap it
                    // Add an extra space to each line because the vertical log screen ignores newlines
                    text_str = if let Some(wrapped) = utils::wrap_text(text, story_view_line_width) {
                        wrapped.join(" \n").to_il2cpp_string()
                    }
                    else {
                        text.to_il2cpp_string()
                    }
                }
                else {
                    let size = StoryTimelineTextClipData::get_Size(this);
                    text_str = if size == StoryTimelineTextClipData::FontSize_Default {
                        if let Some(fitted) = utils::wrap_fit_text(text, line_width, line_count, font_size) {
                            fitted.to_il2cpp_string()
                        }
                        else {
                            text.to_il2cpp_string()
                        }
                    }
                    else {
                        // not doing anything with text of other sizes for now...
                        text.to_il2cpp_string()
                    };
                }
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
        TITLE_FIELD = get_field_from_name(StoryTimelineData, c"Title");
        BLOCKLIST_FIELD = get_field_from_name(StoryTimelineData, c"BlockList");
    }
}