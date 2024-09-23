use std::path::Path;

use crate::{core::Hachimi, il2cpp::{symbols::get_method_addr, ext::StringExt, types::*}};

type GetMovieFilePathFn = extern "C" fn(this: *mut Il2CppObject, movie_file: *mut Il2CppString) -> *mut Il2CppString;
extern "C" fn GetMovieFilePath(this: *mut Il2CppObject, movie_file: *mut Il2CppString) -> *mut Il2CppString {
    let orig_fn = get_orig_fn!(GetMovieFilePath, GetMovieFilePathFn);

    let movie_file_str = unsafe { (*movie_file).as_utf16str().to_string() };
    let mut rel_replace_path = Path::new("movies").join(movie_file_str.to_ascii_lowercase());
    rel_replace_path.set_extension("usm");

    let localized_data = Hachimi::instance().localized_data.load();
    let Some(replace_path) = localized_data.get_assets_path(&rel_replace_path) else {
        return orig_fn(this, movie_file);
    };

    if let Ok(metadata) = std::fs::metadata(&replace_path) {
        if metadata.is_file() {
            return replace_path.to_str()
                .map(|s| s.to_il2cpp_string())
                .unwrap_or_else(|| orig_fn(this, movie_file));
        }
    }

    orig_fn(this, movie_file)
}

pub fn init(Cute_Cri_Assembly: *const Il2CppImage) {
    get_class_or_return!(Cute_Cri_Assembly, "Cute.Cri", MovieManager);

    let GetMovieFilePath_addr = get_method_addr(MovieManager, c"GetMovieFilePath", 1);

    new_hook!(GetMovieFilePath_addr, GetMovieFilePath);
}