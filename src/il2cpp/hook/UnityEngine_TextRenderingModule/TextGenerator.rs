use crate::{core::{ext::StringExt, template, Hachimi}, il2cpp::{symbols::get_method_addr, types::*}};

/**
 * UnityEngine.TextRenderingModule.dll - UnityEngine::TextGenerator::PopulateWithErrors
 *
 * A variant of TextGenerator::Populate that the game exclusively uses to render text.
 *
 * DEV NOTE: Due to the lack of usage context, it should be considered a fallback for when there are
 * strings that can't be modified directly using another hook.
 */
type PopulateWithErrorsFn = extern "C" fn(
    this: *mut Il2CppObject, str: *mut Il2CppString,
    settings: *mut TextGenerationSettings_t, context: *mut Il2CppObject
) -> bool;
extern "C" fn PopulateWithErrors(
    this: *mut Il2CppObject, str_: *mut Il2CppString,
    settings: *mut TextGenerationSettings_t, context: *mut Il2CppObject
) -> bool {
    let orig_fn = get_orig_fn!(PopulateWithErrors, PopulateWithErrorsFn);
    let hashed_dict = &Hachimi::instance().localized_data.load().hashed_dict;
    if hashed_dict.is_empty() {
        return orig_fn(this, str_, settings, context);
    }

    let hash = unsafe { (*str_).hash() };
    if let Some(text) = hashed_dict.get(&hash) {
        orig_fn(this, text.to_il2cpp_string(), settings, context)
    }
    else {
        let str = unsafe { (*str_).to_utf16str() };

        // Only try to evaluate a template if it looks like one
        let new_str = if str.as_slice().contains(&36) { // 36 = dollar sign ($)
            let mut context = TemplateContext {
                settings: unsafe { settings.as_mut().unwrap() }
            };
            Hachimi::instance().template_parser
                .eval_with_context(&str.to_string(), &mut context)
                .to_il2cpp_string()
        }
        else {
            str_
        };
        orig_fn(this, new_str, settings, context)
    }
}

struct TemplateContext<'a> {
    settings: &'a mut TextGenerationSettings_t
}

impl<'a> template::Context for TemplateContext<'a> {
    fn on_filter_eval(&mut self, name: &str, args: &[template::Token]) -> Option<String> {
        // Extra filters to modify the text generation settings
        match name {
            "nb" => {
                self.settings.horizontalOverflow = HorizontalWrapMode_Overflow;
                self.settings.generateOutOfBounds = true;
            }
            
            "anchor" => {
                // Anchor values:
                // 1  2  3
                // 4  5  6
                // 7  8  9
                // Example: $(anchor 6) = middle right
                let value = args.get(0)?;
                let template::Token::NumberLit(anchor_num) = *value else {
                    return None;
                };
                let anchor = (anchor_num as i32) - 1;
                if anchor < 0 || anchor > 8 {
                    return None;
                }
                self.settings.textAnchor = anchor;
            }

            "scale" => {
                // Example: $(scale 80) = scale font size to 80%
                let value = args.get(0)?;
                let template::Token::NumberLit(percentage) = value else {
                    return None;
                };
                self.settings.fontSize = (self.settings.fontSize as f64 * (percentage / 100.0)) as i32;
            }

            "ho" => {
                // $(ho 0) or $(ho 1)
                let value = args.get(0)?;
                let template::Token::NumberLit(overflow_num) = *value else {
                    return None;
                };
                let overflow = overflow_num as i32;
                if overflow != 0 && overflow != 1 {
                    return None;
                }
                self.settings.horizontalOverflow = overflow;
            }

            "vo" => {
                // $(vo 0) or $(vo 1)
                let value = args.get(0)?;
                let template::Token::NumberLit(overflow_num) = *value else {
                    return None;
                };
                let overflow = overflow_num as i32;
                if overflow != 0 && overflow != 1 {
                    return None;
                }
                self.settings.verticalOverflow = overflow;
            }

            _ => return None
        }

        Some(String::new())
    }
}

// Context that ignores TextGenerator filters
pub struct IgnoreTGFiltersContext();

impl template::Context for IgnoreTGFiltersContext {
    fn on_filter_eval(&mut self, _name: &str, _args: &[template::Token]) -> Option<String> {
        match _name {
            "nb" | "anchor" | "scale" => Some(String::new()),
            _ => None
        }
    }
}

pub fn init(UnityEngine_TextRenderingModule: *const Il2CppImage) {
    get_class_or_return!(UnityEngine_TextRenderingModule, UnityEngine, TextGenerator);

    let PopulateWithErrors_addr = get_method_addr(TextGenerator, c"PopulateWithErrors", 3);

    new_hook!(PopulateWithErrors_addr, PopulateWithErrors);
}