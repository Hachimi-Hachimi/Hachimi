use crate::{
    core::Hachimi,
    il2cpp::{hook::UnityEngine_UI::Text, symbols::{get_field_from_name, get_field_object_value, get_method_addr}, types::*}
};

static mut DESCTEXT_FIELD: *mut FieldInfo = 0 as _;
fn get__descText(this: *mut Il2CppObject) -> *mut Il2CppObject {
    get_field_object_value(this, unsafe { DESCTEXT_FIELD })
}

type UpdateItemFn = extern "C" fn(this: *mut Il2CppObject, skill_info: *mut Il2CppObject, is_plate_effect_enable: bool);
extern "C" fn UpdateItem(this: *mut Il2CppObject, skill_info: *mut Il2CppObject, is_plate_effect_enable: bool) {
    get_orig_fn!(UpdateItem, UpdateItemFn)(this, skill_info, is_plate_effect_enable);

    if let Some(mult) = Hachimi::instance().localized_data.load().config.skill_list_item_desc_font_size_multiplier {
        let desc_text = get__descText(this);
        let font_size = Text::get_fontSize(desc_text);
        Text::set_fontSize(desc_text, (font_size as f32 * mult).round() as i32);
    }
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, PartsSingleModeSkillListItem);

    let UpdateItem_addr = get_method_addr(PartsSingleModeSkillListItem, c"UpdateItem", 2);

    new_hook!(UpdateItem_addr, UpdateItem);

    unsafe {
        DESCTEXT_FIELD = get_field_from_name(PartsSingleModeSkillListItem, c"_descText");
    }
}