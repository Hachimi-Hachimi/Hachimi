use crate::{
    core::{ext::StringExt, Hachimi},
    il2cpp::{
        hook::UnityEngine_TextRenderingModule::TextGenerator::IgnoreTGFiltersContext,
        symbols::get_method_addr,
        types::*
    }
};

type PlayTypeWriteFn = extern "C" fn(this: *mut Il2CppObject, message: *mut Il2CppString);
extern "C" fn PlayTypeWrite(this: *mut Il2CppObject, mut message: *mut Il2CppString) {
    let message_utf16 = unsafe { (*message).to_utf16str() };
    if message_utf16.as_slice().contains(&36) { // 36 = dollar sign ($)
        message = Hachimi::instance().template_parser
            .eval_with_context(&message_utf16.to_string(), &mut IgnoreTGFiltersContext())
            .to_il2cpp_string()
    }

    get_orig_fn!(PlayTypeWrite, PlayTypeWriteFn)(this, message);
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, TrainingParamChangePlate);

    let PlayTypeWrite_addr = get_method_addr(TrainingParamChangePlate, c"PlayTypeWrite", 1);

    new_hook!(PlayTypeWrite_addr, PlayTypeWrite);
}