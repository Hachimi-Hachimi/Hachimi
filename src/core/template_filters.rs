use super::{template, Hachimi};

pub static LIST: [(&str, template::Filter); 2] = [
    ("plural", plural),
    ("ordinal", ordinal)
];

// $(plural n 'plural_type_0' 'plural_type_1' ...)
fn plural(args: &[template::Token]) -> Option<String> {
    if args.len() < 2 { return None; }

    if let template::Token::NumberLit(n) = args[0] {
        let hachimi = Hachimi::instance();
        let plural_type = 1 + hachimi.localized_data.load().plural_form.resolve(n as u64);
        let Some(res) = args.get(plural_type) else {
            return None;
        };
        if let template::Token::StringLit(str) = res {
            return Some(str.replace("$", &n.to_string()));
        }
    }

    None
}

// $(ordinal n)
fn ordinal(args: &[template::Token]) -> Option<String> {
    if let template::Token::NumberLit(n) = args[0] {
        let localized_data = Hachimi::instance().localized_data.load();
        let i = localized_data.ordinal_form.resolve(n as u64);
        let Some(ordinal_type) = localized_data.config.ordinal_types.get(i) else {
            return None;
        };
        return Some(ordinal_type.replace("$", &n.to_string()));
    }

    None
}