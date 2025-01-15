use crate::{
    core::Hachimi,
    il2cpp::{
        ext::{Il2CppStringExt, StringExt}, hook::UnityEngine_UI::CanvasScaler, symbols::{get_method_addr, get_method_overload_addr, Array, SingletonLike}, types::*
    }
};

static mut CLASS: *mut Il2CppClass = 0 as _;
pub fn class() -> *mut Il2CppClass {
    unsafe { CLASS }
}

pub fn instance() -> *mut Il2CppObject {
    let Some(singleton) = SingletonLike::new(class()) else {
        return 0 as _;
    };
    singleton.instance()
}

static mut GETCANVASSCALERLIST_ADDR: usize = 0;
impl_addr_wrapper_fn!(GetCanvasScalerList, GETCANVASSCALERLIST_ADDR, Array, this: *mut Il2CppObject);

pub fn apply_ui_scale() {
    let config = Hachimi::instance().config.load();

    #[allow(unused_mut)]
    let mut scale = config.ui_scale;

    #[cfg(target_os = "windows")]
    {
        if let Some((width, height)) = crate::windows::utils::get_scaling_res() {
            if width < height {
                scale *= width as f32 / 1080.0;
            }
            else {
                scale *= height as f32 / 1080.0;
            }
        }
    }

    let ui_manager = instance();
    let canvas_scaler_list = GetCanvasScalerList(ui_manager);
    for scaler in unsafe { canvas_scaler_list.as_slice().iter() } {
        #[cfg(target_os = "android")]
        {
            let res = CanvasScaler::get_m_ReferenceResolution(*scaler);
            unsafe {
                (*res).x /= scale;
                (*res).y /= scale;
            }
        }
        
        #[cfg(target_os = "windows")]
        CanvasScaler::set_scaleFactor(*scaler, scale);
    }
}

type SetHeaderTitleTextFn = extern "C" fn(this: *mut Il2CppObject, text: *mut Il2CppString, guide_id: i32);
extern "C" fn SetHeaderTitleText(this: *mut Il2CppObject, text_: *mut Il2CppString, guide_id: i32) {
    let text = unsafe { (*text_).as_utf16str() };

    // The title text (aka the purple ribbon on the top left of the screen) doesn't run
    // through TextGenerator, so we have to evaluate templates here (by emptying any filter exprs)
    let new_text = if text.as_slice().contains(&36) { // 36 = dollar sign ($)
        Hachimi::instance().template_parser
            .remove_filters(&text.to_string())
            .to_il2cpp_string()
    }
    else {
        text_
    };

    get_orig_fn!(SetHeaderTitleText, SetHeaderTitleTextFn)(this, new_text, guide_id)
}

#[cfg(target_os = "windows")]
type ChangeResizeUIForPCFn = extern "C" fn(this: *mut Il2CppObject, width: i32, height: i32);
#[cfg(target_os = "windows")]
extern "C" fn ChangeResizeUIForPC(this: *mut Il2CppObject, width: i32, height: i32) {
    use super::GraphicSettings;

    get_orig_fn!(ChangeResizeUIForPC, ChangeResizeUIForPCFn)(this, width, height);
    // Recreate the render texture so it scales with the resolution
    if Hachimi::instance().config.load().windows.resolution_scaling.is_not_default() {
        CreateRenderTextureFromScreen(this);
        GraphicSettings::Update3DRenderTexture(GraphicSettings::instance());
    }
    apply_ui_scale();
}

#[cfg(target_os = "android")]
extern "C" fn WaitBootSetup_MoveNext(enumerator: *mut Il2CppObject) -> bool {
    use crate::il2cpp::symbols::MoveNextFn;
    let moved = get_orig_fn!(WaitBootSetup_MoveNext, MoveNextFn)(enumerator);
    if !moved {
        apply_ui_scale();
    }
    moved
}

#[cfg(target_os = "android")]
type WaitBootSetupFn = extern "C" fn(this: *mut Il2CppObject) -> crate::il2cpp::symbols::IEnumerator;
#[cfg(target_os = "android")]
extern "C" fn WaitBootSetup(this: *mut Il2CppObject) -> crate::il2cpp::symbols::IEnumerator {
    let enumerator = get_orig_fn!(WaitBootSetup, WaitBootSetupFn)(this);
    if Hachimi::instance().config.load().ui_scale == 1.0 { return enumerator; }

    if let Err(e) = enumerator.hook_move_next(WaitBootSetup_MoveNext) {
        error!("Failed to hook enumerator: {}", e);
    }

    enumerator
}

#[cfg(target_os = "windows")]
static mut CREATERENDERTEXTUREFROMSCREEN_ADDR: usize = 0;
#[cfg(target_os = "windows")]
impl_addr_wrapper_fn!(CreateRenderTextureFromScreen, CREATERENDERTEXTUREFROMSCREEN_ADDR, (), this: *mut Il2CppObject);

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, UIManager);

    let SetHeaderTitleText_addr = get_method_overload_addr(UIManager, "SetHeaderTitleText",
        &[Il2CppTypeEnum_IL2CPP_TYPE_STRING, Il2CppTypeEnum_IL2CPP_TYPE_VALUETYPE]);

    new_hook!(SetHeaderTitleText_addr, SetHeaderTitleText);

    #[cfg(target_os = "windows")]
    {
        let ChangeResizeUIForPC_addr = get_method_addr(UIManager, c"ChangeResizeUIForPC", 2);

        new_hook!(ChangeResizeUIForPC_addr, ChangeResizeUIForPC);
    }

    #[cfg(target_os = "android")]
    {
        let WaitBootSetup_addr = get_method_addr(UIManager, c"WaitBootSetup", 0);

        new_hook!(WaitBootSetup_addr, WaitBootSetup);
    }

    unsafe {
        CLASS = UIManager;
        GETCANVASSCALERLIST_ADDR = get_method_addr(UIManager, c"GetCanvasScalerList", 0);

        #[cfg(target_os = "windows")]
        { CREATERENDERTEXTUREFROMSCREEN_ADDR = get_method_addr(UIManager, c"CreateRenderTextureFromScreen", 0); }
    }
}