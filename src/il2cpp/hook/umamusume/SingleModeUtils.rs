use crate::{core::{template, Hachimi}, il2cpp::{ext::{Il2CppStringExt, StringExt}, symbols::get_method_addr, types::*}};

use super::{Localize, MasterSingleModeTurn::SingleModeTurn, TextId};

type GetMonthTextByTurnFn = extern "C" fn(turn_set_id: i32, turn: i32) -> *mut Il2CppString;
extern "C" fn GetMonthTextByTurn(turn_set_id: i32, turn: i32) -> *mut Il2CppString {
    if let Some(format) = &Hachimi::instance().localized_data.load().config.month_text_format {
        struct Context {
            turn: *mut Il2CppObject
        }

        impl template::Context for Context {
            fn on_filter_eval(&mut self, _name: &str, args: &[template::Token]) -> Option<String> {
                if args.len() != 0 { return None; }
                match _name {
                    "month" => {
                        let text = GetMonthText(SingleModeTurn::get_Month(self.turn));
                        Some(unsafe { (*text).as_utf16str().to_string() })
                    },
                    "half" => {
                        let half = SingleModeTurn::get_Half(self.turn);
                        let text = Localize::Get(TextId::from_name(
                            if half == 1 { "SingleMode0237" } else { "SingleMode0238" }
                        ));
                        Some(unsafe { (*text).as_utf16str().to_string() })
                    },
                    _ => None
                }
            }
        }

        let turn = GetMasterTurn(turn_set_id, turn);
        return Hachimi::instance().template_parser
            .eval_with_context(format, &mut Context { turn })
            .to_il2cpp_string()
    }

    get_orig_fn!(GetMonthTextByTurn, GetMonthTextByTurnFn)(turn_set_id, turn)
}

// MasterSingleModeTurn.SingleModeTurn
static mut GETMASTERTURN_ADDR: usize = 0;
impl_addr_wrapper_fn!(GetMasterTurn, GETMASTERTURN_ADDR, *mut Il2CppObject, turn_set_id: i32, turn: i32);

static mut GETMONTHTEXT_ADDR: usize = 0;
impl_addr_wrapper_fn!(GetMonthText, GETMONTHTEXT_ADDR, *mut Il2CppString, month: i32);

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, SingleModeUtils);

    let GetMonthTextByTurn_addr = get_method_addr(SingleModeUtils, c"GetMonthTextByTurn", 2);

    new_hook!(GetMonthTextByTurn_addr, GetMonthTextByTurn);

    unsafe {
        GETMASTERTURN_ADDR = get_method_addr(SingleModeUtils, c"GetMasterTurn", 2);
        GETMONTHTEXT_ADDR = get_method_addr(SingleModeUtils, c"GetMonthText", 1);
    }
}