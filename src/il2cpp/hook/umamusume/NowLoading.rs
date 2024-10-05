use crate::{core::{utils::truncate_text_il2cpp, Hachimi}, il2cpp::{hook::UnityEngine_UI::Text, symbols::{get_field_from_name, get_field_object_value, get_method_addr}, types::*}};

static mut _COMICTITLE_FIELD: *mut FieldInfo = 0 as _;
fn get__comicTitle(this: *mut Il2CppObject) -> *mut Il2CppObject {
    get_field_object_value(this, unsafe { _COMICTITLE_FIELD })
}

const COMIC_TITLE_LINE_WIDTH: usize = 23;

type SetupLoadingTipsFn = extern "C" fn(this: *mut Il2CppObject);
extern "C" fn SetupLoadingTips(this: *mut Il2CppObject) {
    get_orig_fn!(SetupLoadingTips, SetupLoadingTipsFn)(this);

    if Hachimi::instance().localized_data.load().config.now_loading_comic_title_ellipsis {
        let comic_title = get__comicTitle(this);
        if comic_title.is_null() { return; }

        let text = Text::get_text(comic_title);
        if text.is_null() { return; }

        if let Some(new_text) = truncate_text_il2cpp(text, COMIC_TITLE_LINE_WIDTH, true) {
            Text::set_horizontalOverflow(comic_title, 1);
            Text::set_text(comic_title, new_text);
        }
    }
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, NowLoading);

    let SetupLoadingTips_addr = get_method_addr(NowLoading, c"SetupLoadingTips", 0);

    new_hook!(SetupLoadingTips_addr, SetupLoadingTips);

    unsafe {
        _COMICTITLE_FIELD = get_field_from_name(NowLoading, c"_comicTitle");
    }
}