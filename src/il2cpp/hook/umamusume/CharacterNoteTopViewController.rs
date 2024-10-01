use crate::{
    core::{hachimi::UITextConfig, Hachimi},
    il2cpp::{
    ext::StringExt, hook::{UnityEngine_CoreModule::{Component, GameObject}, UnityEngine_TextRenderingModule::TextAnchor, UnityEngine_UI::Text}, symbols::get_method_addr, types::*
}};

use super::{ButtonCommon, CharacterNoteTopView, TextCommon, ViewControllerBase};

type InitializeViewFn = extern "C" fn(this: *mut Il2CppObject) -> *mut Il2CppObject;
extern "C" fn InitializeView(this: *mut Il2CppObject) -> *mut Il2CppObject {
    let view = ViewControllerBase::GetView(this);
    let config = &Hachimi::instance().localized_data.load().config;

    if let Some(config) = config.character_note_top_gallery_button.as_ref() {
        let gallery_button = CharacterNoteTopView::get_ButtonGallery(view);
        apply_gallery_button_config(gallery_button, config);
    }

    if let Some(config) = config.character_note_top_talk_gallery_button.as_ref() {
        let talk_gallery_button = CharacterNoteTopView::get_ButtonTalkGallery(view);
        apply_gallery_button_config(talk_gallery_button, config);
    }

    get_orig_fn!(InitializeView, InitializeViewFn)(this)
}

fn apply_gallery_button_config(button: *mut Il2CppObject, config: &UITextConfig) {
    let target_text = ButtonCommon::get_TargetText(button);

    if let Some(text) = config.text.as_ref() {
        let game_object = Component::get_gameObject(button);
        let text_objects = GameObject::GetComponentsInChildren(game_object, TextCommon::type_object(), true);

        let empty_str = "".to_il2cpp_string();
        for text_object in unsafe { text_objects.as_slice().iter() } {
            if *text_object != target_text {
                Text::set_text(*text_object, empty_str);
            }
        }

        Text::set_horizontalOverflow(target_text, 1);
        Text::set_alignment(target_text, TextAnchor::UpperLeft);
        Text::set_text(target_text, text.to_il2cpp_string());
    }

    if let Some(font_size) = config.font_size {
        Text::set_fontSize(target_text, font_size);
    }

    if let Some(line_spacing) = config.line_spacing {
        Text::set_lineSpacing(target_text, line_spacing);
    }
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, CharacterNoteTopViewController);

    let InitializeView_addr = get_method_addr(CharacterNoteTopViewController, c"InitializeView", 0);

    new_hook!(InitializeView_addr, InitializeView);
}