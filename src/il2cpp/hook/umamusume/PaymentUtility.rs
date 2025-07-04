use rust_i18n::t;
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_CLOSE};

use crate::{core::{gui::SimpleYesNoDialog, Gui, Hachimi}, il2cpp::{symbols::get_method_addr, types::*}, windows::{steamworks, wnd_hook}};

type StartPurchaseFn = extern "C" fn(this: *mut Il2CppObject, store_product_id: *mut Il2CppString, is_alert_agree: bool);
extern "C" fn StartPurchase(this: *mut Il2CppObject, store_product_id: *mut Il2CppString, is_alert_agree: bool) {
    // check it again cuz it might change later
    if steamworks::is_overlay_conflicting(&Hachimi::instance()) {
        let mut gui = Gui::instance().unwrap().lock().unwrap();
        gui.show_window(Box::new(SimpleYesNoDialog::new(
            &t!("steam_overlay_conflict_dialog.title"),
            &t!("steam_overlay_conflict_dialog.content"),
            |yes| {
                if yes {
                    let hachimi = Hachimi::instance();
                    let mut config = hachimi.config.load().as_ref().clone();
                    config.disable_gui_once = true;
                    _ = hachimi.save_and_reload_config(config);
                    unsafe {
                        _ = PostMessageW(wnd_hook::get_target_hwnd(), WM_CLOSE, None, None);
                    }
                }
            }
        )));
    }
    get_orig_fn!(StartPurchase, StartPurchaseFn)(this, store_product_id, is_alert_agree);
}

pub fn init(umamusume: *const Il2CppImage) {
    // dont need this hook if the overlay isn't conflicting
    if !steamworks::is_overlay_conflicting(&Hachimi::instance()) {
        return;
    }

    get_class_or_return!(umamusume, Gallop, PaymentUtility);

    let StartPurchase_addr = get_method_addr(PaymentUtility, c"StartPurchase", 2);

    new_hook!(StartPurchase_addr, StartPurchase);
}