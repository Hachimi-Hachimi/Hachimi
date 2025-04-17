#[macro_use] extern crate log;
#[macro_use] extern crate cstr;

rust_i18n::i18n!("assets/locales", fallback = "en");

#[macro_use] pub mod core;
pub mod il2cpp;

/** Android **/
#[cfg(target_os = "android")]
mod android;

#[cfg(target_os = "android")]
use android::{log_impl, game_impl, hachimi_impl, gui_impl, symbols_impl, interceptor_impl};

/** Windows **/
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
use windows::{log_impl, game_impl, hachimi_impl, gui_impl, symbols_impl, interceptor_impl};