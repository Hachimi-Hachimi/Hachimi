#[macro_use] extern crate log;
#[macro_use] extern crate cstr;

#[macro_use] pub mod core;
pub mod il2cpp;

/** Android **/
#[cfg(target_os = "android")]
mod android;

#[cfg(target_os = "android")]
use android::{log_impl, game_impl, hachimi_impl, gui_impl};