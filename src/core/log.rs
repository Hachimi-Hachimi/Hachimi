use crate::log_impl;
use super::Hachimi;

pub fn init(hachimi: &Hachimi) {
    let debug_mode = hachimi.config.load().debug_mode;
    let filter_level = if debug_mode {
        log::LevelFilter::Debug
    }
    else {
        log::LevelFilter::Info
    };

    log_impl::init(filter_level);
}