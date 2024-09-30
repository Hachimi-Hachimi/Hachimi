use std::ptr::null_mut;

use serde::Deserialize;
use widestring::Utf16Str;

use crate::{
    core::{ext::Utf16StringExt, utils::{self, IsolateTags}, Hachimi}, 
    il2cpp::{
        ext::StringExt, hook::{umamusume::{StoryTimelineCharaTrackData, StoryTimelineClipData}, UnityEngine_AssetBundleModule::AssetBundle::ASSET_PATH_PREFIX}, symbols::{get_field_from_name, get_field_object_value, get_field_value, set_field_object_value, set_field_value, IList}, types::*
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

static mut TYPEWRITECOUNTPERSECOND_FIELD: *mut FieldInfo = null_mut();
fn get_TypewriteCountPerSecond(this: *mut Il2CppObject) -> i32 {
    get_field_value(this, unsafe { TYPEWRITECOUNTPERSECOND_FIELD })
}

fn set_TypewriteCountPerSecond(this: *mut Il2CppObject, value: i32) {
    set_field_value(this, unsafe { TYPEWRITECOUNTPERSECOND_FIELD }, &value);
}

static mut LENGTH_FIELD: *mut FieldInfo = null_mut();
fn set_Length(this: *mut Il2CppObject, value: i32) {
    set_field_value(this, unsafe { LENGTH_FIELD }, &value);
}

// (Aliases are there for tlg compatibility)
#[derive(Deserialize)]
struct StoryTimelineDataDict {
    #[serde(alias = "Title")]
    title: Option<String>,

    #[serde(alias = "TextBlockList")]
    #[serde(default)]
    text_block_list: Vec<TextBlockDict>,

    #[serde(default)]
    no_wrap: bool
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

    new_clip_length: Option<i32>
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

    let hachimi = Hachimi::instance();
    let mut tcps = get_TypewriteCountPerSecond(this) as f32;
    let tcps_mult = hachimi.config.load().story_tcps_multiplier;
    if tcps_mult != 1.0 {
        tcps = (tcps * tcps_mult).round();
        set_TypewriteCountPerSecond(this, tcps as i32);
    }

    let base_path = name[ASSET_PATH_PREFIX.len()..].path_basename();
    let dict_path = base_path.to_string() + ".json";

    let localized_data = hachimi.localized_data.load();
    let Some(dict): Option<StoryTimelineDataDict> = localized_data.load_assets_dict(Some(&dict_path)) else {
        // Clip length adjustment independent of story patching
        // No need to adjust length if speed is faster
        if tcps_mult < 1.0 {
            adjust_clips_length_with_tcps(this, tcps);
        }
        return;
    };
    debug!("{}", dict_path);

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

    let mut total_len = 0;
    let mut total_len_changed = false;
    for (mut i, block_data) in block_list.iter().enumerate() {
        let orig_block_len = StoryTimelineBlockData::get_BlockLength(block_data);
        total_len += orig_block_len;

        // First block is always empty, skip over it
        if i == 0 { continue; }
        i -= 1;

        let Some(text_block_dict) = dict.text_block_list.get(i) else {
            warn!("text block {} not found in dict: {}", i, dict_path);
            break;
        };

        let Some(clip_data) = StoryTimelineBlockData::get_text_clip(block_data) else {
            continue;
        };

        if let Some(name) = &text_block_dict.name {
            StoryTimelineTextClipData::set_Name(clip_data, name.to_il2cpp_string());
        }

        if let Some(text) = &text_block_dict.text {
            let mut modified_text = None;
            if !dict.no_wrap {
                if is_story_view {
                    // Sizing tags are not used at all in main stories, simply wrap it
                    // Add an extra space to each line because the vertical log screen ignores newlines
                    if let Some(wrapped) = utils::wrap_text(text, story_view_line_width) {
                        modified_text = Some(wrapped.join(" \n"));
                    }
                }
                else {
                    let size = StoryTimelineTextClipData::get_Size(this);
                    if size == StoryTimelineTextClipData::FontSize_Default {
                        if let Some(fitted) = utils::wrap_fit_text(text, line_width, line_count, font_size) {
                            modified_text = Some(fitted);
                        }
                    }
                    // not doing anything with text of other sizes for now...
                }
            }
            let new_text = modified_text.as_ref().unwrap_or(text);
            StoryTimelineTextClipData::set_Text(clip_data, new_text.to_il2cpp_string());

            // Adjust clip length
            if localized_data.config.auto_adjust_story_clip_length || text_block_dict.new_clip_length.is_some() {
                let new_clip_len = text_block_dict.new_clip_length.unwrap_or_else(|| {
                    let text_len = IsolateTags::new(new_text).fold(0, |total_len, (s, is_not_tag)| 
                        if is_not_tag { total_len + s.chars().count() } else { total_len }
                    );
                    // Everything else down here is in the unit of frames at 30fps
                    let typewrite_len = get_typewrite_length(text_len, tcps);
                    return StoryTimelineTextClipData::get_WaitFrame(clip_data) +
                        typewrite_len.max(StoryTimelineTextClipData::get_VoiceLength(clip_data));
                });

                let orig_clip_len = StoryTimelineClipData::get_ClipLength(clip_data);
                if new_clip_len > orig_clip_len {
                    let new_block_len = apply_clip_length(
                        clip_data, orig_clip_len, new_clip_len,
                        block_data, orig_block_len
                    );
                    let block_len_diff = new_block_len - orig_block_len;
                    total_len += block_len_diff;
                    total_len_changed = true;
                }
            }
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

    if total_len_changed {
        set_Length(this, total_len);
    }
}

fn get_typewrite_length(text_len: usize, tcps: f32) -> i32 {
    (text_len as f32 / tcps * 30.0).round() as i32 // len / cps * fps
}

fn adjust_clips_length_with_tcps(this: *mut Il2CppObject, tcps: f32) {
    let Some(block_list) = IList::new(get_BlockList(this)) else {
        return;
    };
    let mut block_list_iter = block_list.iter();

    // First block is always empty, no need to adjust length
    let Some(first_block_data) = block_list_iter.next() else {
        return;
    };
    let mut total_len = StoryTimelineBlockData::get_BlockLength(first_block_data);

    for block_data in block_list_iter {
        let orig_block_len = StoryTimelineBlockData::get_BlockLength(block_data);
        let Some(clip_data) = StoryTimelineBlockData::get_text_clip(block_data) else {
            total_len += orig_block_len;
            continue;
        };
        let text = StoryTimelineTextClipData::get_Text(clip_data);

        total_len += if text.is_null() {
            orig_block_len
        }
        else {
            let orig_clip_len = StoryTimelineClipData::get_ClipLength(clip_data);
            let new_clip_len = get_typewrite_length(unsafe { (*text).as_utf16str().chars().count() }, tcps);

            if new_clip_len > orig_clip_len {
                apply_clip_length(clip_data, orig_clip_len, new_clip_len, block_data, orig_block_len)
            }
            else {
                orig_block_len
            }
        }
    }

    set_Length(this, total_len);
}

/// Returns new block length
fn apply_clip_length(
    clip_data: *mut Il2CppObject, orig_clip_len: i32, new_clip_len: i32,
    block_data: *mut Il2CppObject, orig_block_len: i32
) -> i32 {
    StoryTimelineClipData::set_ClipLength(clip_data, new_clip_len);
    let new_block_len = StoryTimelineClipData::get_StartFrame(clip_data) + new_clip_len + 1;
    StoryTimelineBlockData::set_BlockLength(block_data, new_block_len);

    let clip_len_diff = new_clip_len - orig_clip_len;

    // Adjust anim lengths
    if let Some(chara_track_list) = <IList>::new(StoryTimelineBlockData::get_CharacterTrackList(block_data)) {
        for chara_track_data in chara_track_list.iter() {
            for motion_track_data in StoryTimelineCharaTrackData::motion_track_data_values(chara_track_data) {
                let Some(clip_list) = <IList>::new(StoryTimelineTrackData::get_ClipList(motion_track_data)) else {
                    continue;
                };
                let Some(clip_data) = clip_list.get(clip_list.count() - 1) else {
                    continue;
                };

                let orig_motion_clip_len = StoryTimelineClipData::get_ClipLength(clip_data);
                let new_motion_clip_len = orig_motion_clip_len + clip_len_diff;
                StoryTimelineClipData::set_ClipLength(clip_data, new_motion_clip_len);
            }
        }
    }

    // Adjust screen effect lengths
    if let Some(se_track_list) = <IList>::new(StoryTimelineBlockData::get_ScreenEffectTrackList(block_data)) {
        for se_track_data in se_track_list.iter() {
            let Some(clip_list) = <IList>::new(StoryTimelineTrackData::get_ClipList(se_track_data)) else {
                continue;
            };
            let Some(clip_data) = clip_list.get(clip_list.count() - 1) else {
                continue;
            };

            let start_frame = StoryTimelineClipData::get_StartFrame(clip_data);
            let orig_se_clip_len = StoryTimelineClipData::get_ClipLength(clip_data);
            // if it extends to the end of the block
            if start_frame + orig_se_clip_len < orig_block_len {
                continue;
            }

            let new_se_clip_len = orig_se_clip_len + clip_len_diff;
            StoryTimelineClipData::set_ClipLength(clip_data, new_se_clip_len);
        }
    }

    new_block_len
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, StoryTimelineData);

    unsafe {
        CLASS = StoryTimelineData;
        TITLE_FIELD = get_field_from_name(StoryTimelineData, c"Title");
        BLOCKLIST_FIELD = get_field_from_name(StoryTimelineData, c"BlockList");
        TYPEWRITECOUNTPERSECOND_FIELD = get_field_from_name(StoryTimelineData, c"TypewriteCountPerSecond");
        LENGTH_FIELD = get_field_from_name(StoryTimelineData, c"Length");
    }
}